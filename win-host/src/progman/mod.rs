mod worker;
pub use worker::*;

use crate::WinApiError;
use thiserror::Error;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, PSTR, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowA, FindWindowExA, SendMessageTimeoutA, SMTO_NORMAL, WNDENUMPROC,
};

const PROG_MAN_CREATE_WORKER_W_MSG: u32 = 0x052C;

/// ProgMan window interface
#[derive(Debug)]
pub struct ProgMan {
    window: HWND,
}

impl ProgMan {
    /// Attempts to create a new ProgMan instance by finding the window for management.
    pub fn new() -> Result<ProgMan, Error> {
        let prog_man_window = unsafe { FindWindowA("Progman", None) };

        if prog_man_window.0 == 0 {
            Err(Error::NotFound)
        } else {
            Ok(Self {
                window: prog_man_window,
            })
        }
    }

    /// Gets or creates the WorkerW window.
    pub fn get_or_create_worker(&self) -> Result<Worker, Error> {
        let mut message_result = 0usize;
        let result = unsafe {
            SendMessageTimeoutA(
                self.window,
                PROG_MAN_CREATE_WORKER_W_MSG,
                None,
                None,
                SMTO_NORMAL,
                1000,
                &mut message_result,
            )
        };

        if result == LRESULT(0) {
            return Err(Error::SendMessageFailure(WinApiError::last()));
        }

        let mut worker_w: Option<HWND> = None;
        let result = unsafe {
            EnumWindows(
                Some(Self::enum_windows_callback),
                LPARAM(&mut worker_w as *mut _ as _),
            )
        }
        .as_bool();

        let worker_w = match (result, worker_w) {
            (true, None) => return Err(Error::NotFound),
            (false, None) => return Err(Error::EnumWindows(WinApiError::last())),
            (false, Some(window)) => window,
            _ => unreachable!("Invalid combination of enumeration result and found window"),
        };

        Ok(unsafe { Worker::new(worker_w) })
    }

    extern "system" fn enum_windows_callback(window: HWND, out: LPARAM) -> BOOL {
        let worker_w: &mut Option<HWND> = unsafe { std::mem::transmute(out) };
        let def_view_window =
            unsafe { FindWindowExA(window, None, "SHELLDLL_DefView", None) };

        if def_view_window.0 != 0 {
            let maybe_worker_w =
                unsafe { FindWindowExA(None, window, "WorkerW", None) };

            if maybe_worker_w.0 != 0 {
                worker_w.replace(maybe_worker_w);
                return false.into();
            }
        }

        true.into()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("window handle not found")]
    NotFound,

    #[error("failed to send window message: {0}")]
    SendMessageFailure(WinApiError),

    #[error("failed to enumerate windows: {0}")]
    EnumWindows(WinApiError),
}
