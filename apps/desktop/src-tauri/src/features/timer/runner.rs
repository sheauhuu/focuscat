use super::{
    timer::{AutoAdvance, TimerStatus},
    types::{TimerDto, TimerState, TimerUpdatedEvent},
};
use crate::features::audio::{audio, types::SoundId};
use crate::features::settings::types::AppSettingsState;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[cfg(target_os = "macos")]
use crate::app::tray::TrayState;

// MARK: - Timer Runner

/// Handle to control the timer runner thread.
pub struct TimerRunner {
    stop_flag: Arc<AtomicBool>,
}

impl TimerRunner {
    /// Start timer runner in background thread.
    pub fn start(app: AppHandle) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        thread::spawn(move || {
            run_timer_loop(app, stop_flag_clone);
        });

        return Self { stop_flag };
    }

    /// Stop the timer runner.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}

// MARK: - Timer Loop

fn run_timer_loop(app: AppHandle, stop_flag: Arc<AtomicBool>) {
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        // Get sleep duration based on speed
        let (sleep_duration, speed) = {
            let timer_state = match app.try_state::<TimerState>() {
                Some(s) => s,
                None => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            };

            let timer = timer_state.lock().unwrap();

            if timer.status != TimerStatus::Running {
                drop(timer);
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            let speed = app
                .try_state::<AppSettingsState>()
                .map(|s| s.lock().unwrap().developer.timer_speed)
                .unwrap_or(1)
                .max(1);

            (Duration::from_millis(1000 / speed as u64), speed)
        };

        thread::sleep(sleep_duration);

        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        // Get state for tick
        let state_handle = match app.try_state::<TimerState>() {
            Some(s) => s,
            None => continue,
        };

        let mut timer = state_handle.lock().unwrap();

        if timer.status != TimerStatus::Running {
            continue;
        }

        let previous_overtime = timer.overtime_seconds;
        let was_in_overtime = previous_overtime > 0;

        // Count down or count overtime
        if timer.remaining_seconds > 0 {
            timer.remaining_seconds -= 1;
        } else {
            timer.overtime_seconds += 1;
        }

        let entered_overtime = !was_in_overtime && timer.overtime_seconds > 0;
        let next_work_seconds = timer.next_work_seconds;
        let session_id = timer.active_session_id();
        let settings = app.state::<AppSettingsState>().lock().unwrap().clone();
        let action = timer.auto_advance(&settings, previous_overtime);
        let snapshot = TimerDto::from(&*timer);
        drop(timer);

        // Native tray updates can wait for the main thread, so release timer state first.
        if speed == 1 {
            audio::play(&app, SoundId::Tick);
        }
        if entered_overtime {
            audio::play(&app, SoundId::Complete);
        }
        let _ = TimerUpdatedEvent(snapshot.clone()).emit(&app);
        #[cfg(target_os = "macos")]
        TrayState::set_timer(&app, Some(snapshot.remaining_seconds));

        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        if let Some(action) = action {
            match action {
                AutoAdvance::Pomodoro | AutoAdvance::ProgressiveWork => {
                    let result = tauri::async_runtime::block_on(async {
                        if action == AutoAdvance::Pomodoro {
                            return super::commands::advance_pomodoro_timer(
                                app.clone(),
                                app.state(),
                                app.state(),
                                app.state(),
                                app.state(),
                                None,
                            )
                            .await;
                        }
                        return super::commands::advance_progressive_timer(
                            app.clone(),
                            app.state(),
                            app.state(),
                            app.state(),
                            app.state(),
                            super::commands::ProgressiveSessionType::Work,
                            next_work_seconds,
                            None,
                            None,
                        )
                        .await;
                    });
                    if let Err(error) = result {
                        eprintln!("[Timer] Automatic advance failed: {error}");
                    } else {
                        // Advancing starts a new runner with its own stop flag.
                        break;
                    }
                }
                _ => {
                    let path = match action {
                        AutoAdvance::PomodoroSetup => "/window/main/pomodoro/setup?advance=true",
                        AutoAdvance::ProgressiveSetup => {
                            "/window/main/progressive/setup?advance=true"
                        }
                        AutoAdvance::ProgressiveRating => "/window/main/progressive/rating",
                        _ => unreachable!(),
                    };
                    let app_handle = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        let state = app_handle.state::<TimerState>();
                        let timer = state.lock().unwrap();
                        if timer.active_session_id() != session_id
                            || timer.status != TimerStatus::Running
                            || timer.overtime_seconds == 0
                        {
                            return;
                        }
                        drop(timer);
                        if let Err(error) =
                            crate::app::presentation::show_timer_flow(&app_handle, path)
                        {
                            eprintln!("[Timer] Failed to show session flow: {error}");
                        }
                    });
                }
            }
        }
    }
}
