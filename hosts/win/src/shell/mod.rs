//! Windows shell integration.

use std::ffi::CString;
use std::ops::Not;

use thiserror::Error;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, PSTR, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GetWindowLongPtrA,
    PeekMessageA, PostQuitMessage, RegisterClassA, SetWindowLongPtrA, TranslateMessage,
    UnregisterClassA, GWLP_USERDATA, MSG, PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WM_NCCREATE, WM_QUIT, WNDCLASSA,
};

use integration::*;

use crate::util::WinApiError;

mod integration;

/// The window class name for the shell integration window.
const WINDOW_CLASS_NAME: &str = "SnowlandWinHostShellIntegration";

extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    log::debug!("Processing window message, window = {:?}, message = 0x{:X}, w_param = {:?}, l_param = {:?}", window, message, w_param, l_param);

    // Protect against panics
    std::panic::catch_unwind(|| {
        match message {
            WM_NCCREATE => {
                let integration = match ShellIntegration::new(window) {
                    Ok(v) => v,
                    Err(err) => {
                        log::error!("Failed to create shell integration: {}", err);
                        return LRESULT(0);
                    }
                };

                // Window created, create the shell integration instance for this window
                let mut integration = Box::new(integration);

                let result = integration.callback(message, w_param, l_param);
                unsafe {
                    SetWindowLongPtrA(window, GWLP_USERDATA, Box::into_raw(integration) as isize)
                };

                Some(result)
            }

            WM_DESTROY => {
                // Window destroyed, re-wrap the shell integration into a box which will then be dropped
                let integration = unsafe {
                    let integration =
                        SetWindowLongPtrA(window, GWLP_USERDATA, 0) as *mut ShellIntegration;
                    integration
                        .is_null()
                        .not()
                        .then(|| Box::from_raw(integration))
                };

                integration.and_then(|mut integration| {
                    integration
                        .handles(window)
                        .then(|| integration.callback(message, w_param, l_param))
                })
            }

            message => {
                // Other message, try to acquire the shell integration
                let integration = unsafe {
                    let integration =
                        GetWindowLongPtrA(window, GWLP_USERDATA) as *mut ShellIntegration;
                    integration.as_mut()
                };

                integration.and_then(|integration| {
                    integration
                        .handles(window)
                        .then(|| integration.callback(message, w_param, l_param))
                })
            }
        }
        .and_then(|result| match result {
            Ok(v) => Some(v),
            Err(err) => {
                log::error!("Failed to handle window message: {}", err);
                None
            }
        })
        .unwrap_or_else(|| unsafe { DefWindowProcA(window, message, w_param, l_param) })
    })
    .unwrap_or_else(|err| {
        log::error!("Panic while handling window message: {:?}", err);
        log::error!("Posting quit message and running default window procedure!");
        unsafe { PostQuitMessage(1) };

        unsafe { DefWindowProcA(window, message, w_param, l_param) }
    })
}

/// Helper window in order to provide message processing.
///
/// This window will never be visible or directly interacted with by the user, but is required
/// to for example interact properly with the system tray.
#[derive(Debug)]
pub struct ShellIntegrationWindow {
    h_instance: HINSTANCE,
    window: HWND,
}

impl ShellIntegrationWindow {
    /// Creates the shell integration window.
    pub fn new() -> Result<Self, Error> {
        let h_instance = unsafe { GetModuleHandleA(None) };

        let mut window_class_name = CString::new(WINDOW_CLASS_NAME).unwrap().into_bytes();

        let class = WNDCLASSA {
            hInstance: h_instance,
            lpszClassName: PSTR(window_class_name.as_mut_ptr()),
            lpfnWndProc: Some(window_procedure),
            ..Default::default()
        };

        let window_class = unsafe { RegisterClassA(&class) };
        if window_class == 0 {
            return Err(Error::ClassRegistrationFailed(WinApiError::from_win32()));
        }

        // This creates a very basic window which is not visible.
        let window = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(0),
                WINDOW_CLASS_NAME,
                WINDOW_CLASS_NAME,
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                h_instance,
                std::ptr::null(),
            )
        };

        if window.0 == 0 {
            return Err(Error::WindowCreationFailed(WinApiError::from_win32()));
        }

        Ok(ShellIntegrationWindow { h_instance, window })
    }

    /// Process all messages in the message queue and then returns control.
    pub fn process_messages(&self) -> bool {
        let mut msg = MSG::default();

        unsafe {
            while PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                log::debug!("Dispatching message: {:?}", msg);
                if msg.message == WM_QUIT {
                    return false;
                }

                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            true
        }
    }
}

impl Drop for ShellIntegrationWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.window);
            UnregisterClassA(WINDOW_CLASS_NAME, self.h_instance);
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to register window class: {0}")]
    ClassRegistrationFailed(WinApiError),

    #[error("failed to create window: {0}")]
    WindowCreationFailed(WinApiError),
}
