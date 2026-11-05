#[cfg(target_os = "macos")]
mod ffi;
mod keep_awake;

pub use keep_awake::{DisplaySleepAssertion, KeepAwakeError};

#[cfg(target_os = "macos")]
use ffi::*;

/// Adds the system glass material behind a menu bar window's existing content.
///
/// # Safety
///
/// `window` must point to a live `NSWindow`. Call on the macOS main thread.
#[cfg(target_os = "macos")]
pub unsafe fn apply_menubar_material(window: *mut std::ffi::c_void, corner_radius: f64) -> bool {
    return unsafe { macos_apply_menubar_material(window, corner_radius) };
}

/// Check if input monitoring permission is granted.
///
/// On macOS, this permission is required for global keyboard/mouse event detection.
///
/// Returns `true` on non-macOS platforms.
pub fn is_input_monitoring_enabled() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { macos_is_input_monitoring_enabled() }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}
