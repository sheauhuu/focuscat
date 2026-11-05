/// Keeps the display awake until released or the owning process exits.
pub struct DisplaySleepAssertion {
    #[cfg(target_os = "macos")]
    id: Option<u32>,
}

impl DisplaySleepAssertion {
    pub fn acquire() -> Result<Self, KeepAwakeError> {
        #[cfg(target_os = "macos")]
        {
            let mut id = 0;
            let result = unsafe { crate::ffi::macos_create_display_sleep_assertion(&mut id) };
            if result != 0 {
                return Err(KeepAwakeError::Create(result));
            }
            return Ok(Self { id: Some(id) });
        }

        #[cfg(not(target_os = "macos"))]
        return Err(KeepAwakeError::UnsupportedPlatform);
    }

    pub fn release(&mut self) -> Result<(), KeepAwakeError> {
        #[cfg(target_os = "macos")]
        if let Some(id) = self.id {
            let result = unsafe { crate::ffi::macos_release_display_sleep_assertion(id) };
            if result != 0 {
                return Err(KeepAwakeError::Release(result));
            }
            self.id = None;
        }
        return Ok(());
    }
}

impl Drop for DisplaySleepAssertion {
    fn drop(&mut self) {
        if let Err(error) = self.release() {
            eprintln!("Failed to release display sleep assertion: {error}");
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeepAwakeError {
    #[error("Keeping the screen awake is only supported on macOS")]
    UnsupportedPlatform,
    #[error("Failed to keep the screen awake (IOKit error {0})")]
    Create(i32),
    #[error("Failed to restore display sleep (IOKit error {0})")]
    Release(i32),
}
