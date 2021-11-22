use crate::WinApiError;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;

/// Represents a ProgMan worker window.
#[derive(Debug)]
pub struct Worker {
    window: HWND,
}

impl Worker {
    /// Creates a new ProgMan worker window from an existing window handle.
    ///
    /// # Safety
    /// The caller must guarantee, that the passed handle is a valid handle to
    /// `WorkerW` window from ProgMan.
    pub unsafe fn new(window: HWND) -> Self {
        Self { window }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let destroy_result = unsafe { DestroyWindow(self.window) }.as_bool();
        if !destroy_result {
            log::warn!(
                "Failed to destroy worker window when dropping worker: {}",
                WinApiError::last()
            );
        }
    }
}
