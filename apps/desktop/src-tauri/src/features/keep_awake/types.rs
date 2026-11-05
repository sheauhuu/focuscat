use focuscat_macos::DisplaySleepAssertion;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Default)]
pub struct KeepAwakeState(pub Mutex<Option<DisplaySleepAssertion>>);

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
pub struct KeepAwakeChangedEvent(pub bool);
