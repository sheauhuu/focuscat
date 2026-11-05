use crate::features::session::session::SessionType;
use crate::features::settings::types::AppSettings;

#[derive(Debug, Clone)]
pub struct ProgressivePomodoroMode {
    pub start_work_duration_seconds: u32,
}

impl ProgressivePomodoroMode {
    pub fn from_settings(settings: &AppSettings) -> Self {
        // Use the second suggestion of the first rating (typically "distracted" → 5 min)
        // to encourage starting small, matching the Progressive Pomodoro philosophy.
        let start_seconds = settings
            .timer
            .progressive
            .ratings
            .first()
            .and_then(|r| r.suggestions.get(1))
            .map(|s| s.work_minutes * 60)
            .unwrap_or(5 * 60);
        return Self {
            start_work_duration_seconds: start_seconds,
        };
    }

    pub fn first_session(&self) -> (SessionType, u32) {
        return (
            SessionType::ProgressiveWork,
            self.start_work_duration_seconds,
        );
    }

    /// Progressive work durations come from the user's rating, stored by the timer commands.
    pub fn next_session(
        &self,
        _current: SessionType,
        _completed_work: u32,
    ) -> Option<(SessionType, u32)> {
        return None;
    }
}
