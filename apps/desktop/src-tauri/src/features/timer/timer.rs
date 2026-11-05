use super::modes::TimerMode;
use crate::features::session::session::{Session, SessionEvent, SessionStatus, SessionType};
use crate::features::settings::types::AppSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Timer {
    pub status: TimerStatus,
    pub total_seconds: u32,
    pub remaining_seconds: u32,
    pub overtime_seconds: u32,
    pub sessions_completed: u32,
    pub next_work_seconds: u32,
    pub mode: TimerMode,
    pub session: Option<Session>,
}

impl Timer {
    pub fn from_settings(settings: &AppSettings) -> Self {
        let mode = TimerMode::from_settings(settings);
        let (_session_type, duration_seconds) = mode.first_session();
        return Self {
            status: TimerStatus::Idle,
            total_seconds: duration_seconds,
            remaining_seconds: duration_seconds,
            overtime_seconds: 0,
            sessions_completed: 0,
            next_work_seconds: duration_seconds,
            mode,
            session: None,
        };
    }

    pub fn apply_settings(&mut self, settings: &AppSettings) {
        self.mode = TimerMode::from_settings(settings);
    }

    /// Resolve automatic transitions independently of any frontend window.
    pub fn auto_advance(
        &self,
        settings: &AppSettings,
        previous_overtime: u32,
    ) -> Option<AutoAdvance> {
        if self.status != TimerStatus::Running || self.session.is_none() {
            return None;
        }
        let (enabled, countdown) = match self.mode {
            TimerMode::Pomodoro(_) => (
                settings.timer.pomodoro.auto_advance,
                settings.timer.pomodoro.auto_advance_countdown_seconds,
            ),
            TimerMode::Progressive(_) => (
                settings.timer.progressive.auto_advance,
                settings.timer.progressive.auto_advance_countdown_seconds,
            ),
            TimerMode::Countdown(_) => return None,
        };
        let threshold = countdown.max(1);
        if !enabled || previous_overtime >= threshold || self.overtime_seconds < threshold {
            return None;
        }
        return match self.active_session_type() {
            SessionType::ProgressiveWork => Some(AutoAdvance::ProgressiveRating),
            SessionType::ProgressiveBreak if settings.timer.progressive.show_session_setup => {
                Some(AutoAdvance::ProgressiveSetup)
            }
            SessionType::ProgressiveBreak => Some(AutoAdvance::ProgressiveWork),
            session_type
                if !session_type.is_work() && settings.timer.pomodoro.show_session_setup =>
            {
                Some(AutoAdvance::PomodoroSetup)
            }
            _ => Some(AutoAdvance::Pomodoro),
        };
    }

    /// First session in the queue.
    pub fn first_session(&self) -> (SessionType, u32) {
        return self.mode.first_session();
    }

    /// Next session after completing the current one. None if no next (e.g. countdown done).
    pub fn next_session(
        &self,
        current_session_type: SessionType,
        completed_work: u32,
    ) -> Option<(SessionType, u32)> {
        return self.mode.next_session(current_session_type, completed_work);
    }

    /// Advance the timer by completing the current session and starting the given one.
    pub fn advance_to_session(
        &mut self,
        new_session: Session,
        next_duration_seconds: u32,
        completed_was_work: bool,
        now: i64,
    ) {
        if let Some(s) = &mut self.session {
            s.status = SessionStatus::Completed;
            s.ended_at = Some(now);
            s.add_event(SessionEvent::Completed { timestamp: now });
        }
        if completed_was_work {
            self.sessions_completed += 1;
        }
        self.session = Some(new_session);
        self.total_seconds = next_duration_seconds;
        self.remaining_seconds = next_duration_seconds;
        self.overtime_seconds = 0;
        self.status = TimerStatus::Running;
    }

    /// Active session id, if any.
    pub fn active_session_id(&self) -> Option<i64> {
        return self.session.as_ref().map(|s| s.id);
    }

    /// Active session’s type if running/paused, else the first session type for this mode.
    pub fn active_session_type(&self) -> SessionType {
        return self
            .session
            .as_ref()
            .map(|s| s.session_type)
            .unwrap_or_else(|| self.mode.first_session().0);
    }

    /// Clear active session and set timer to idle; mode and countdown from settings.
    pub fn reset_to_idle(&mut self, settings: &AppSettings) {
        self.apply_settings(settings);
        self.session = None;
        self.status = TimerStatus::Idle;
        let (_session_type, duration_seconds) = self.first_session();
        self.total_seconds = duration_seconds;
        self.remaining_seconds = duration_seconds;
        self.overtime_seconds = 0;
        self.sessions_completed = 0;
        self.next_work_seconds = duration_seconds;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoAdvance {
    Pomodoro,
    ProgressiveWork,
    PomodoroSetup,
    ProgressiveSetup,
    ProgressiveRating,
}

impl Default for Timer {
    fn default() -> Self {
        return Self::from_settings(&AppSettings::default());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
}

#[cfg(test)]
mod tests {
    use super::{AutoAdvance, Timer, TimerStatus};
    use crate::features::{
        session::session::{Session, SessionType},
        settings::types::{AppSettings, BlockThreshold, TimerModeEnum},
    };

    fn running_timer(settings: &AppSettings, session_type: SessionType) -> Timer {
        let mut timer = Timer::from_settings(settings);
        timer.session = Some(Session::new(
            1,
            session_type,
            60,
            None,
            BlockThreshold::None,
            0,
        ));
        timer.status = TimerStatus::Running;
        timer.remaining_seconds = 0;
        timer.overtime_seconds = 5;
        return timer;
    }

    #[test]
    fn auto_advance_fires_once_at_the_overtime_threshold() {
        let mut settings = AppSettings::default();
        settings.timer.pomodoro.auto_advance = true;
        let mut timer = running_timer(&settings, SessionType::PomodoroWork);

        assert_eq!(
            timer.auto_advance(&settings, 4),
            Some(AutoAdvance::Pomodoro)
        );
        assert_eq!(timer.auto_advance(&settings, 5), None);
        timer.status = TimerStatus::Paused;
        assert_eq!(timer.auto_advance(&settings, 4), None);
        timer.status = TimerStatus::Running;
        settings.timer.pomodoro.auto_advance = false;
        assert_eq!(timer.auto_advance(&settings, 4), None);
    }

    #[test]
    fn work_setup_is_requested_only_when_returning_from_a_break() {
        let mut settings = AppSettings::default();
        settings.timer.pomodoro.auto_advance = true;
        settings.timer.pomodoro.show_session_setup = true;
        let work = running_timer(&settings, SessionType::PomodoroWork);
        let rest = running_timer(&settings, SessionType::PomodoroShortBreak);

        assert_eq!(work.auto_advance(&settings, 4), Some(AutoAdvance::Pomodoro));
        assert_eq!(
            rest.auto_advance(&settings, 4),
            Some(AutoAdvance::PomodoroSetup)
        );
    }

    #[test]
    fn progressive_work_requests_rating_and_break_respects_session_setup() {
        let mut settings = AppSettings::default();
        settings.timer.timer_mode = TimerModeEnum::Progressive;
        settings.timer.progressive.auto_advance = true;
        let work = running_timer(&settings, SessionType::ProgressiveWork);
        let rest = running_timer(&settings, SessionType::ProgressiveBreak);

        assert_eq!(
            work.auto_advance(&settings, 4),
            Some(AutoAdvance::ProgressiveRating)
        );
        assert_eq!(
            rest.auto_advance(&settings, 4),
            Some(AutoAdvance::ProgressiveWork)
        );
        settings.timer.progressive.show_session_setup = true;
        assert_eq!(
            rest.auto_advance(&settings, 4),
            Some(AutoAdvance::ProgressiveSetup)
        );
    }

    #[test]
    fn zero_countdown_advances_on_the_first_overtime_tick() {
        let mut settings = AppSettings::default();
        settings.timer.pomodoro.auto_advance = true;
        settings.timer.pomodoro.auto_advance_countdown_seconds = 0;
        let mut timer = running_timer(&settings, SessionType::PomodoroWork);
        timer.overtime_seconds = 1;

        assert_eq!(
            timer.auto_advance(&settings, 0),
            Some(AutoAdvance::Pomodoro)
        );
        assert_eq!(timer.auto_advance(&settings, 1), None);
    }

    #[test]
    fn changing_the_selected_mode_does_not_change_active_auto_advance() {
        let mut settings = AppSettings::default();
        settings.timer.pomodoro.auto_advance = true;
        let pomodoro = running_timer(&settings, SessionType::PomodoroWork);

        settings.timer.timer_mode = TimerModeEnum::Countdown;
        assert_eq!(
            pomodoro.auto_advance(&settings, 4),
            Some(AutoAdvance::Pomodoro)
        );

        let countdown = running_timer(&settings, SessionType::Countdown);
        settings.timer.timer_mode = TimerModeEnum::Pomodoro;
        assert_eq!(countdown.auto_advance(&settings, 4), None);
    }
}
