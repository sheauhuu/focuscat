//! Keeps the screen awake independently of the timer and visible windows.

pub mod commands;
pub mod types;

use tauri::{App, Manager};

pub fn setup(app: &App) {
    app.manage(types::KeepAwakeState::default());
}
