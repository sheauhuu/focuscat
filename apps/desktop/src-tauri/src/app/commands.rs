use super::window::Window;
use crate::environment::configs::app::{AppConfig, AppDistribution};
use serde::Serialize;
use specta::Type;

// MARK: - App Info

#[tauri::command]
#[specta::specta]
pub fn get_app_info() -> AppInfoDto {
    let base_version = env!("CARGO_PKG_VERSION");
    let (stage, suffix) = if cfg!(debug_assertions) {
        (Stage::Dev, "-dev")
    } else {
        (Stage::Prod, "")
    };

    AppInfoDto {
        version: format!("v{}{}", base_version, suffix),
        stage,
        distribution: AppConfig::distribution(),
    }
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoDto {
    pub version: String,
    pub stage: Stage,
    pub distribution: AppDistribution,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Dev,
    Prod,
}

// MARK: - Window Commands

#[tauri::command]
#[specta::specta]
pub fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    Window::Main.show(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn show_main_window_at_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    Window::Main
        .show_at_path(&app, &path)
        .map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn show_cat_window(app: tauri::AppHandle) -> Result<(), String> {
    Window::Cat.show(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    Window::Main.hide(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn navigate_main_window_to_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    Window::Main
        .navigate_to_path(&app, &path)
        .map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn hide_cat_window(app: tauri::AppHandle) -> Result<(), String> {
    Window::Cat.hide(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn show_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    super::menubar::hide(&app);
    Window::Settings.show(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn show_settings_window_at_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    super::menubar::hide(&app);
    Window::Settings
        .show_at_path(&app, &path)
        .map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn hide_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    Window::Settings.hide(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn navigate_settings_window_to_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    Window::Settings
        .navigate_to_path(&app, &path)
        .map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn show_activity_window(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    super::menubar::hide(&app);
    Window::Activity.show(&app).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn show_activity_window_at_session(
    app: tauri::AppHandle,
    session_id: i32,
) -> Result<(), String> {
    let path = format!("/window/activity/{}", session_id);

    #[cfg(target_os = "macos")]
    super::menubar::hide(&app);
    Window::Activity
        .show_at_path(&app, &path)
        .map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
#[specta::specta]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
#[specta::specta]
pub fn show_menubar_at_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return super::menubar::show_at_path(&app, &path);
    #[cfg(not(target_os = "macos"))]
    return Err("Menu bar presentation is only available on macOS".into());
}

#[tauri::command]
#[specta::specta]
pub fn hide_menubar(app: tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    super::menubar::hide(&app);
}

#[tauri::command]
#[specta::specta]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
