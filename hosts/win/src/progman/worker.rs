use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

use crate::util::WinApiError;

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

    /// Retrieves the handle of the worker window.
    pub fn get_handle(&self) -> HWND {
        self.window
    }

    /// Retrieves the size of the window.
    pub fn get_size(&self) -> Result<(u64, u64), WinApiError> {
        let mut rect = Default::default();

        if !unsafe { GetWindowRect(self.window, &mut rect) }.as_bool() {
            Err(WinApiError::from_win32())
        } else {
            Ok((
                (rect.right - rect.left) as u64,
                (rect.bottom - rect.top) as u64,
            ))
        }
    }

    // /// Sets another window as a child of the worker window.
    // pub fn reparent_other_as_child(&self, other: HWND) {
    //     unsafe { SetParent(other, self.window) };
    // }
}
