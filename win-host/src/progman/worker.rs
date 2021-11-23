use crate::graphics::Graphics;
use crate::WinApiError;
use thiserror::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{GetDCEx, DCX_CACHE, DCX_LOCKWINDOWUPDATE, DCX_WINDOW};
use windows::Win32::UI::WindowsAndMessaging::{DestroyWindow, SetParent};

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

    /// Sets another window as a child of the worker window.
    pub fn reparent_other_as_child(&self, other: HWND) {
        unsafe { SetParent(other, self.window) };
    }
}
