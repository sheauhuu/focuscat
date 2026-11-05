use super::types::{KeepAwakeChangedEvent, KeepAwakeState};
use focuscat_macos::DisplaySleepAssertion;
use tauri::{AppHandle, State};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
pub fn get_keep_awake(state: State<'_, KeepAwakeState>) -> bool {
    state.0.lock().unwrap().is_some()
}

#[tauri::command]
#[specta::specta]
pub fn set_keep_awake(
    app: AppHandle,
    state: State<'_, KeepAwakeState>,
    enabled: bool,
) -> Result<(), String> {
    let mut assertion = state.0.lock().unwrap();
    if enabled {
        if assertion.is_none() {
            *assertion = Some(DisplaySleepAssertion::acquire().map_err(|error| error.to_string())?);
        }
    } else if let Some(active) = assertion.as_mut() {
        active.release().map_err(|error| error.to_string())?;
        *assertion = None;
    }

    let _ = KeepAwakeChangedEvent(assertion.is_some()).emit(&app);
    return Ok(());
}
