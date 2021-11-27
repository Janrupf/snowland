use std::ffi::CString;
use std::mem::MaybeUninit;

use thiserror::Error;
use windows::Win32::Foundation::{CHAR, HWND, LPARAM, LRESULT, POINT, PWSTR, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::Controls::RichEdit::WM_CONTEXTMENU;
use windows::Win32::UI::Controls::{LoadIconMetric, LIM_SMALL};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconA, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_SETVERSION, NOTIFYICONDATAA, NOTIFYICONDATAA_0, NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuA, CreatePopupMenu, DefWindowProcA, DestroyMenu, GetCursorPos, PostQuitMessage,
    SetForegroundWindow, TrackPopupMenu, HICON, HMENU, MF_STRING, TPM_NONOTIFY, TPM_RETURNCMD,
    WM_CREATE, WM_DESTROY, WM_USER,
};

use snowland_universal::control::ControlMessage;

use crate::shell::messenger::{HostToIntegrationMessage, IntegrationMessenger};
use crate::WinApiError;

/// Shell integration of Snowland.
///
/// This is managed by the shell integration window and has all the control
/// to provide integration with the Windows system shell.
pub struct ShellIntegration {
    messenger: IntegrationMessenger,
    window: HWND,
    menu: HMENU,
    icon_data: NOTIFYICONDATAA,
}

/// Window message sent by the Snowland host thread to notify that a message is in the messenger.
pub const WM_SNOWLAND_MESSENGER: u32 = WM_USER + 1;

/// Window message sent by the Snowland notification system tay.
const WM_SNOWLAND_NOTIFICATION: u32 = WM_USER + 2;

// Modified version of:
// https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer/29530566
macro_rules! back_to_enum {
    (#[repr($repr_t:ident)] $(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        #[repr($repr_t)]
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<$repr_t> for $name {
            type Error = $repr_t;

            fn try_from(v: $repr_t) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as $repr_t => Ok($name::$vname),)*
                    x => Err(x),
                }
            }
        }
    }
}

back_to_enum! {
    #[repr(usize)]
    #[derive(Debug)]
    enum MenuItem {
        Exit,
        ShowUI,
    }
}

impl ShellIntegration {
    /// Creates the shell integration.
    ///
    /// At this point the window has been fully created.
    pub fn new(messenger: IntegrationMessenger, window: HWND) -> Result<Self, Error> {
        messenger.window_created(window);

        let menu = unsafe {
            let menu = CreatePopupMenu();

            if menu.0 == 0 {
                return Err(Error::MenuCreationFailed(WinApiError::from_win32()));
            }

            AppendMenuA(menu, MF_STRING, MenuItem::Exit as _, "Exit");
            AppendMenuA(menu, MF_STRING, MenuItem::ShowUI as _, "Show UI");

            menu
        };

        let icon_data = NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
            hWnd: window,
            uCallbackMessage: WM_SNOWLAND_NOTIFICATION,
            uID: 1,

            // Show the icon and tooltip and use a GUID to identify the icon calling the window
            // with a window message.
            uFlags: NIF_ICON | NIF_TIP | NIF_SHOWTIP | NIF_MESSAGE,
            hIcon: Self::load_icon(1),
            szTip: Self::make_string("Snowland"),

            // Set to version 4
            Anonymous: NOTIFYICONDATAA_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },

            ..Default::default()
        };

        Ok(Self {
            messenger,
            window,
            menu,
            icon_data,
        })
    }

    /// Determines whether this shell integration handles messages for the specified window.
    pub fn handles(&self, window: HWND) -> bool {
        self.window == window
    }

    /// Processes a window message.
    pub fn callback(
        &mut self,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> Result<LRESULT, Error> {
        match message {
            WM_CREATE => self.create().map(|()| LRESULT(0)),
            WM_DESTROY => self.destroy().map(|()| LRESULT(0)),
            WM_SNOWLAND_MESSENGER => self
                .process_host_message(*unsafe { Box::from_raw(w_param.0 as _) })
                .map(|()| LRESULT(0)),
            WM_SNOWLAND_NOTIFICATION => self
                .process_notification_message(w_param, l_param)
                .map(|()| LRESULT(0)),
            _ => Ok(unsafe { DefWindowProcA(self.window, message, w_param, l_param) }),
        }
    }

    /// Creates and initializes the integration.
    fn create(&mut self) -> Result<(), Error> {
        let icon_added = unsafe { Shell_NotifyIconA(NIM_ADD, &self.icon_data) }.as_bool();
        if !icon_added {
            return Err(Error::NotificationIconNotAdded);
        }

        let version_set = unsafe { Shell_NotifyIconA(NIM_SETVERSION, &self.icon_data) }.as_bool();
        if !version_set {
            return Err(Error::NotificationVersionNotSupported);
        }

        Ok(())
    }

    /// Destroys the integration and releases all resources.
    fn destroy(&mut self) -> Result<(), Error> {
        unsafe { Shell_NotifyIconA(NIM_DELETE, &self.icon_data) };

        Ok(())
    }

    /// Processes a message received from the host thread.
    fn process_host_message(&mut self, message: HostToIntegrationMessage) -> Result<(), Error> {
        match message {
            HostToIntegrationMessage::QuitLoop => unsafe { PostQuitMessage(0) },
            HostToIntegrationMessage::Control(msg) => self.process_control_message(msg)?,
        }

        Ok(())
    }

    /// Processes a notification message received from the shell.
    fn process_notification_message(
        &mut self,
        _w_param: WPARAM,
        l_param: LPARAM,
    ) -> Result<(), Error> {
        let message = (l_param.0 & 0xFFFF) as u32;

        log::debug!("Received notification with message = {}", message);

        if message == WM_CONTEXTMENU {
            let mut cursor_pos = POINT::default();
            if !unsafe { GetCursorPos(&mut cursor_pos) }.as_bool() {
                log::warn!("Failed to get cursor pos!");
            }

            let click_result = unsafe {
                SetForegroundWindow(self.window);

                TrackPopupMenu(
                    self.menu,
                    TPM_RETURNCMD | TPM_NONOTIFY,
                    cursor_pos.x,
                    cursor_pos.y,
                    0,
                    self.window,
                    std::ptr::null(),
                )
            }
            .0 as usize;

            let click_result = match MenuItem::try_from(click_result) {
                Ok(v) => v,
                Err(err) => {
                    log::warn!("Unknown pop menu click result 0x{:X}", err);
                    return Ok(());
                }
            };

            log::debug!("User clicked menu item {:?}", click_result);

            match click_result {
                MenuItem::Exit => self.messenger.send(ControlMessage::Exit),
                MenuItem::ShowUI => self.messenger.send(ControlMessage::OpenUI),
            }
        }

        Ok(())
    }

    /// Processes a core control message.
    fn process_control_message(&mut self, _message: ControlMessage) -> Result<(), Error> {
        Ok(())
    }

    /// Loads the given resource index as an icon.
    fn load_icon(resource: u32) -> HICON {
        unsafe {
            let hinstance = GetModuleHandleA(None);
            LoadIconMetric(hinstance, PWSTR(resource as *mut _), LIM_SMALL).unwrap()
        }
    }

    /// Creates a fixed length string from a rust string.
    fn make_string<const N: usize>(str: &str) -> [CHAR; N] {
        assert_eq!(
            std::mem::size_of::<[u8; N]>(),
            std::mem::size_of::<[CHAR; N]>()
        );

        let str = CString::new(str).expect("String contained null byte");

        unsafe {
            let mut data = MaybeUninit::<[CHAR; N]>::uninit();
            std::ptr::copy_nonoverlapping(str.as_ptr(), data.as_mut_ptr() as _, N);

            data.assume_init()
        }
    }
}

impl Drop for ShellIntegration {
    fn drop(&mut self) {
        unsafe {
            DestroyMenu(self.menu);
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create popup menu: {0}")]
    MenuCreationFailed(WinApiError),

    #[error("could not add notification icon")]
    NotificationIconNotAdded,

    #[error("notification version 4 not supported by the shell")]
    NotificationVersionNotSupported,
}
