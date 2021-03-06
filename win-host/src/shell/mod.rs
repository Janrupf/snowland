//! Windows shell integration.

use std::ffi::CString;
use std::ops::Not;
use std::ptr::NonNull;
use std::thread::JoinHandle;

use thiserror::Error;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, PSTR, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GetMessageA,
    GetWindowLongPtrA, PostQuitMessage, RegisterClassA, SetWindowLongPtrA, TranslateMessage,
    UnregisterClassA, CREATESTRUCTA, GWLP_USERDATA, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
    WM_NCCREATE, WNDCLASSA,
};

use integration::*;
use snowland_universal::control::ControlMessage;
use snowland_universal::util::{Delayed, DelayedResolver, Notifier};

use crate::WinApiError;

mod integration;

/// Starts the shell integration on a new thread.
pub fn start_shell_integration(
    notifier: Notifier<ControlMessage>,
) -> (Notifier<ControlMessage>, JoinHandle<Result<(), Error>>) {
    let (delayed_notifier, resolver) = Delayed::new();

    let startup_data = ShellIntegrationStartupData::new(notifier, resolver);
    let handle = std::thread::spawn(|| shell_integration_main(startup_data));

    let notifier = delayed_notifier.wait();

    (notifier, handle)
}

/// Starts the shell integration on the current thread.
fn shell_integration_main(data: ShellIntegrationStartupData) -> Result<(), Error> {
    match ShellIntegrationWindow::new(data) {
        Ok(v) => v.run(),
        Err(err) => {
            log::error!("Shell integration failed to start: {}", err);
            Err(err)
        }
    }
}

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
                let create_parameters =
                    unsafe { (NonNull::<CREATESTRUCTA>::new_unchecked(l_param.0 as _)).as_ref() };

                let data = unsafe { Box::from_raw(create_parameters.lpCreateParams as _) };

                let integration = match ShellIntegration::new(*data, window) {
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

/// Bundle of data required to start the shell integration and wire it with the host.
pub(crate) struct ShellIntegrationStartupData {
    /// integration to host notifier
    notifier: Notifier<ControlMessage>,
    /// host to integration notifier, depends on the window to be created
    resolver: DelayedResolver<Notifier<ControlMessage>>,
}

impl ShellIntegrationStartupData {
    /// Compiles the startup data to be ready to be sent to the integration thread.
    fn new(
        notifier: Notifier<ControlMessage>,
        resolver: DelayedResolver<Notifier<ControlMessage>>,
    ) -> Self {
        Self { notifier, resolver }
    }

    /// Resolves this startup data by sending back the host notifier and extracting the integration
    /// notifier.
    pub fn resolve(self, host_notifier: Notifier<ControlMessage>) -> Notifier<ControlMessage> {
        let Self { notifier, resolver } = self;

        resolver.resolve(host_notifier);

        notifier
    }
}

/// Helper window in order to provide message processing.
///
/// This window will never be visible or directly interacted with by the user, but is required
/// to for example interact properly with the system tray.
#[derive(Debug)]
struct ShellIntegrationWindow {
    h_instance: HINSTANCE,
    window: HWND,
}

impl ShellIntegrationWindow {
    /// Creates the shell integration window.
    pub fn new(data: ShellIntegrationStartupData) -> Result<Self, Error> {
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

        // Dispatch a notification about the display configuration
        data.notifier.notify(ControlMessage::UpdateDisplays(
            crate::util::display::get_displays(),
        ));

        let data = Box::into_raw(Box::new(data));

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
                data as _,
            )
        };

        if window.0 == 0 {
            return Err(Error::WindowCreationFailed(WinApiError::from_win32()));
        }

        Ok(ShellIntegrationWindow { h_instance, window })
    }

    /// Runs the windows message loop and processes incoming window messages.
    pub fn run(&self) -> Result<(), Error> {
        let mut msg = MSG::default();

        log::debug!("Starting shell integration loop...");
        unsafe {
            while GetMessageA(&mut msg, None, 0, 0).as_bool() {
                log::debug!("Dispatching message: {:?}", msg);
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
        log::debug!("Shell integration loop finished!");

        Ok(())
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
