use super::runner::TimerRunner;
use super::timer::TimerStatus;
use super::types::{TimerDto, TimerState, TimerUpdatedEvent};
use crate::environment::db::DatabaseState;
use crate::features::session::repository::SessionRepository;
use crate::features::session::session::{SessionEvent, SessionStatus, SessionType};
use crate::features::session::types::{
    SessionChangedEvent, SessionCompletedEvent, SessionSummaryDto,
};
use crate::features::settings::types::{AppSettingsState, BlockThreshold};
use chrono::Utc;
use std::sync::Mutex;
use tauri::{AppHandle, State};
use tauri_specta::Event;

#[cfg(target_os = "macos")]
use crate::app::tray::TrayState;

#[tauri::command]
#[specta::specta]
pub fn get_timer(state: State<'_, TimerState>) -> TimerDto {
    return TimerDto::from(&*state.lock().unwrap());
}

#[tauri::command]
#[specta::specta]
pub async fn start_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    app_settings: State<'_, AppSettingsState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
    input: Option<SessionStartInput>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();
    let SessionStartInput {
        intention,
        profile_ids,
        block_threshold,
    } = input.unwrap_or_default();
    let intention = intention.filter(|s| !s.trim().is_empty());
    let resolved_block_threshold = resolve_block_threshold(&app_settings, block_threshold);

    // Read state
    let (session_type, planned_seconds) = {
        let timer = state.lock().unwrap();
        if timer.status != TimerStatus::Idle {
            return Err("Timer is not idle".to_string());
        }
        let (session_type, _) = timer.first_session();
        // Use remaining_seconds so idle duration adjustments are reflected as planned.
        let planned_seconds = timer.remaining_seconds;
        (session_type, planned_seconds)
    };

    // DB: create session
    let session = SessionRepository::create(
        &db.pool,
        session_type,
        planned_seconds,
        intention.as_deref(),
        resolved_block_threshold,
        now,
    )
    .await
    .map_err(db_err)?;

    // Link profiles if provided
    if let Some(ids) = &profile_ids {
        if !ids.is_empty() {
            SessionRepository::link_profiles(&db.pool, session.id, ids)
                .await
                .map_err(db_err)?;
        }
    }

    // Write state
    let timer = {
        let mut timer = state.lock().unwrap();
        timer.session = Some(session);
        timer.status = TimerStatus::Running;
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(&app);

    #[cfg(target_os = "macos")]
    TrayState::set_timer(&app, Some(timer.remaining_seconds));

    let _ = SessionChangedEvent.emit(&app);
    start_runner(&app, &runner);
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub async fn pause_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();

    // Read state
    let session_id = {
        let timer = state.lock().unwrap();
        if timer.status != TimerStatus::Running {
            return Err("Timer is not running".to_string());
        }
        timer.active_session_id()
    };

    // DB: pause event
    let event = SessionEvent::Paused { timestamp: now };
    if let Some(id) = session_id {
        SessionRepository::insert_event(&db.pool, id, &event)
            .await
            .map_err(db_err)?;
    }

    // Write state
    let timer = {
        let mut timer = state.lock().unwrap();
        if let Some(session) = &mut timer.session {
            session.add_event(event);
        }
        timer.status = TimerStatus::Paused;
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(&app);
    stop_runner(&runner);
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub async fn resume_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();

    // Read state
    let session_id = {
        let timer = state.lock().unwrap();
        if timer.status != TimerStatus::Paused {
            return Err("Timer is not paused".to_string());
        }
        timer.active_session_id()
    };

    // DB: resume event
    let event = SessionEvent::Resumed { timestamp: now };
    if let Some(id) = session_id {
        SessionRepository::insert_event(&db.pool, id, &event)
            .await
            .map_err(db_err)?;
    }

    // Write state
    let timer = {
        let mut timer = state.lock().unwrap();
        if let Some(session) = &mut timer.session {
            session.add_event(event);
        }
        timer.status = TimerStatus::Running;
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(&app);
    start_runner(&app, &runner);
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub async fn reset_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    app_settings: State<'_, AppSettingsState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();

    // Read state
    let (settings, session_data) = {
        let timer = state.lock().unwrap();
        let settings = app_settings.lock().unwrap();
        let session_data = timer
            .session
            .as_ref()
            .map(|s| (s.id, s.compute_actual_seconds(now)));
        ((*settings).clone(), session_data)
    };

    // DB: cancel session
    if let Some((id, actual_seconds)) = session_data {
        SessionRepository::cancel(&db.pool, id, now, actual_seconds)
            .await
            .map_err(db_err)?;
    }

    // Write state
    let timer = {
        let mut timer = state.lock().unwrap();
        timer.reset_to_idle(&settings);
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(&app);

    #[cfg(target_os = "macos")]
    TrayState::set_timer(&app, None);

    let _ = SessionChangedEvent.emit(&app);
    stop_runner(&runner);
    return Ok(());
}

/// Complete the current session and reset timer.
/// Like reset, but marks session as completed instead of cancelled.
#[tauri::command]
#[specta::specta]
pub async fn complete_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    app_settings: State<'_, AppSettingsState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();

    // Read state
    let (settings, session_data) = {
        let timer = state.lock().unwrap();
        let settings = app_settings.lock().unwrap();
        let session_data = timer.session.as_ref().map(|s| {
            (
                s.id,
                s.session_type.as_str().to_string(),
                s.planned_seconds,
                s.compute_actual_seconds(now),
                s.started_at,
            )
        });
        ((*settings).clone(), session_data)
    };

    // DB: complete session
    if let Some(data) = session_data {
        complete_and_emit_session(&db.pool, &app, data, now).await?;
    }

    // Write state
    let timer = {
        let mut timer = state.lock().unwrap();
        timer.reset_to_idle(&settings);
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(&app);

    #[cfg(target_os = "macos")]
    TrayState::set_timer(&app, None);

    let _ = SessionChangedEvent.emit(&app);
    stop_runner(&runner);
    return Ok(());
}

/// Complete the current session and advance to the next in the Pomodoro sequence (work↔break).
/// When the next session is work, intention and profile_ids may be provided.
#[tauri::command]
#[specta::specta]
pub async fn advance_pomodoro_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    app_settings: State<'_, AppSettingsState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
    input: Option<SessionStartInput>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();

    // Read state
    let (session_data, current_session_type, sessions_completed) = {
        let timer = state.lock().unwrap();
        let session_data = timer.session.as_ref().map(|s| {
            (
                s.id,
                s.session_type.as_str().to_string(),
                s.planned_seconds,
                s.compute_actual_seconds(now),
                s.started_at,
            )
        });
        let (current_session_type, sessions_completed) = match &timer.session {
            None => return Err("No active session".to_string()),
            Some(s) => (s.session_type, timer.sessions_completed),
        };
        (session_data, current_session_type, sessions_completed)
    };

    // Next session from mode
    let (next_session_type, next_duration_seconds) = {
        let timer = state.lock().unwrap();
        timer
            .next_session(current_session_type, sessions_completed)
            .unwrap_or_else(|| timer.first_session())
    };

    do_advance(
        &app,
        &state,
        &app_settings,
        &runner,
        &db,
        session_data,
        current_session_type,
        next_session_type,
        next_duration_seconds,
        input.unwrap_or_default(),
        None,
        now,
    )
    .await
}

/// Advance to the specified next Progressive session type and duration.
#[tauri::command]
#[specta::specta]
pub async fn advance_progressive_timer(
    app: AppHandle,
    state: State<'_, TimerState>,
    app_settings: State<'_, AppSettingsState>,
    runner: State<'_, Mutex<Option<TimerRunner>>>,
    db: State<'_, DatabaseState>,
    session_type: ProgressiveSessionType,
    duration_seconds: u32,
    input: Option<SessionStartInput>,
    next_work_seconds: Option<u32>,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();

    let next_session_type = session_type.to_session_type();

    // Read state
    let (session_data, current_session_type) = {
        let timer = state.lock().unwrap();
        let session_data = timer.session.as_ref().map(|s| {
            (
                s.id,
                s.session_type.as_str().to_string(),
                s.planned_seconds,
                s.compute_actual_seconds(now),
                s.started_at,
            )
        });
        let current_session_type = match &timer.session {
            None => return Err("No active session".to_string()),
            Some(s) => s.session_type,
        };
        (session_data, current_session_type)
    };

    do_advance(
        &app,
        &state,
        &app_settings,
        &runner,
        &db,
        session_data,
        current_session_type,
        next_session_type,
        duration_seconds,
        input.unwrap_or_default(),
        next_work_seconds,
        now,
    )
    .await
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ProgressiveSessionType {
    Work,
    Break,
}

impl ProgressiveSessionType {
    fn to_session_type(&self) -> SessionType {
        match self {
            ProgressiveSessionType::Work => SessionType::ProgressiveWork,
            ProgressiveSessionType::Break => SessionType::ProgressiveBreak,
        }
    }
}

/// Shared advance logic: complete the current session and start the specified next one.
async fn do_advance(
    app: &AppHandle,
    state: &State<'_, TimerState>,
    app_settings: &State<'_, AppSettingsState>,
    runner: &State<'_, Mutex<Option<TimerRunner>>>,
    db: &State<'_, DatabaseState>,
    session_data: Option<(i64, String, u32, u32, i64)>,
    current_session_type: SessionType,
    next_session_type: SessionType,
    next_duration_seconds: u32,
    input: SessionStartInput,
    next_work_seconds: Option<u32>,
    now: i64,
) -> Result<(), String> {
    let is_work = current_session_type.is_work();
    let SessionStartInput {
        intention,
        profile_ids,
        block_threshold,
    } = input;
    let intention = intention.filter(|s| !s.trim().is_empty());
    let resolved_block_threshold = resolve_block_threshold(app_settings, block_threshold);

    // DB: complete current session
    if let Some(data) = session_data {
        complete_and_emit_session(&db.pool, app, data, now).await?;
    }

    // DB: create next session
    let intention_for_create = if next_session_type.is_work() {
        intention.as_deref()
    } else {
        None
    };
    let new_session = SessionRepository::create(
        &db.pool,
        next_session_type,
        next_duration_seconds,
        intention_for_create,
        resolved_block_threshold,
        now,
    )
    .await
    .map_err(db_err)?;

    // Link profiles if next is a work session
    if next_session_type.is_work() {
        if let Some(ids) = &profile_ids {
            if !ids.is_empty() {
                SessionRepository::link_profiles(&db.pool, new_session.id, ids)
                    .await
                    .map_err(db_err)?;
            }
        }
    }

    // Write state
    let timer = {
        let mut timer = state.lock().unwrap();
        timer.advance_to_session(new_session, next_duration_seconds, is_work, now);
        if let Some(seconds) = next_work_seconds {
            timer.next_work_seconds = seconds;
        }
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(app);

    #[cfg(target_os = "macos")]
    TrayState::set_timer(app, Some(timer.remaining_seconds));

    let _ = SessionChangedEvent.emit(app);
    restart_runner(app, runner);
    return Ok(());
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", default)]
pub struct SessionStartInput {
    pub intention: Option<String>,
    pub profile_ids: Option<Vec<i32>>,
    pub block_threshold: Option<BlockThreshold>,
}

#[tauri::command]
#[specta::specta]
pub async fn set_timer_duration(
    app: AppHandle,
    state: State<'_, TimerState>,
    db: State<'_, DatabaseState>,
    minutes: u32,
) -> Result<(), String> {
    let now = Utc::now().timestamp_millis();
    let new_seconds = minutes * 60;

    // Read state
    let (is_idle, session_id, old_seconds) = {
        let timer = state.lock().unwrap();
        (
            timer.status == TimerStatus::Idle,
            timer.active_session_id(),
            timer.total_seconds,
        )
    };

    // Calculate delta (how much time was added/removed)
    let delta_seconds = new_seconds as i64 - old_seconds as i64;

    // DB operation (only if active and time was added)
    if !is_idle && delta_seconds > 0 {
        let event = SessionEvent::Extended {
            timestamp: now,
            seconds: delta_seconds as u32,
        };
        if let Some(id) = session_id {
            SessionRepository::insert_event(&db.pool, id, &event)
                .await
                .map_err(db_err)?;
        }

        // Write state (session)
        let mut timer = state.lock().unwrap();
        if let Some(session) = &mut timer.session {
            session.add_event(event);
        }
    }

    // Write state (timer)
    let timer = {
        let mut timer = state.lock().unwrap();
        timer.total_seconds = new_seconds;
        timer.remaining_seconds = new_seconds;
        timer.overtime_seconds = 0;
        timer.clone()
    };

    let _ = TimerUpdatedEvent(TimerDto::from(&timer)).emit(&app);

    #[cfg(target_os = "macos")]
    if timer.status != TimerStatus::Idle {
        TrayState::set_timer(&app, Some(timer.remaining_seconds));
    }

    return Ok(());
}

// MARK: - Helpers

fn resolve_block_threshold(
    app_settings: &State<'_, AppSettingsState>,
    block_threshold: Option<BlockThreshold>,
) -> BlockThreshold {
    return block_threshold.unwrap_or_else(|| app_settings.lock().unwrap().focus.block_threshold);
}

/// Complete session in DB and emit SessionCompletedEvent.
async fn complete_and_emit_session(
    db: &sqlx::SqlitePool,
    app: &AppHandle,
    session_data: (i64, String, u32, u32, i64),
    now: i64,
) -> Result<(), String> {
    let (id, session_type, planned, actual, started_at) = session_data;
    SessionRepository::complete(db, id, now, actual)
        .await
        .map_err(db_err)?;

    let _ = SessionCompletedEvent(SessionSummaryDto {
        id: id as i32,
        session_type,
        status: SessionStatus::Completed,
        planned_seconds: planned,
        actual_seconds: Some(actual),
        intention: None,
        started_at: started_at as f64,
        ended_at: Some(now as f64),
    })
    .emit(app);

    return Ok(());
}

fn db_err(e: sqlx::Error) -> String {
    return format!("Database error: {}", e);
}

fn start_runner(app: &AppHandle, runner: &State<'_, Mutex<Option<TimerRunner>>>) {
    let mut guard = runner.lock().unwrap();
    if guard.is_some() {
        guard.as_ref().unwrap().stop();
    }
    *guard = Some(TimerRunner::start(app.clone()));
}

fn stop_runner(runner: &State<'_, Mutex<Option<TimerRunner>>>) {
    let mut guard = runner.lock().unwrap();
    if let Some(r) = guard.take() {
        r.stop();
    }
}

fn restart_runner(app: &AppHandle, runner: &State<'_, Mutex<Option<TimerRunner>>>) {
    let mut guard = runner.lock().unwrap();
    if let Some(r) = guard.take() {
        r.stop();
    }
    *guard = Some(TimerRunner::start(app.clone()));
}
