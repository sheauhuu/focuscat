use std::ffi::c_void;
use swift_rs::swift;

// Permissions
swift!(pub fn macos_is_input_monitoring_enabled() -> bool);

swift!(pub fn macos_apply_menubar_material(window: *mut c_void, corner_radius: f64) -> bool);

swift!(pub fn macos_create_display_sleep_assertion(assertion_id: *mut u32) -> i32);
swift!(pub fn macos_release_display_sleep_assertion(assertion_id: u32) -> i32);
