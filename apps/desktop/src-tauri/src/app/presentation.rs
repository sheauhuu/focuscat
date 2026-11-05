//! Routes automatic session flows without interrupting the floating cat.

use super::window::Window;
use tauri::AppHandle;

pub fn show_timer_flow(app: &AppHandle, path: &str) -> Result<(), String> {
    let main_visible = Window::Main
        .get(app)
        .is_some_and(|window| window.is_visible().unwrap_or(false));
    #[cfg(target_os = "macos")]
    if super::menubar::is_visible(app) || !main_visible {
        return super::menubar::show_at_path(app, path);
    }

    let cat_visible = Window::Cat
        .get(app)
        .is_some_and(|window| window.is_visible().unwrap_or(false));
    let path = if !main_visible && cat_visible {
        format!(
            "{path}{}returnTargetKind=cat",
            if path.contains('?') { "&" } else { "?" }
        )
    } else {
        path.to_string()
    };
    if !main_visible {
        Window::Cat.hide(app).map_err(|e| e.to_string())?;
    }
    Window::Main
        .show_at_path(app, &path)
        .map_err(|e| e.to_string())?;
    return Ok(());
}
