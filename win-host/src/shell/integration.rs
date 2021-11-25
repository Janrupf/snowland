use std::ffi::CString;
use std::mem::MaybeUninit;

use thiserror::Error;
use windows::core::GUID;
use windows::Win32::Foundation::{CHAR, HWND, LPARAM, LRESULT, POINT, PWSTR, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::Controls::RichEdit::WM_CONTEXTMENU;
use windows::Win32::UI::Controls::{LoadIconMetric, LIM_SMALL};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconA, NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_SETVERSION, NOTIFYICONDATAA, NOTIFYICONDATAA_0, NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuA, CreatePopupMenu, DefWindowProcA, DestroyMenu, GetCursorPos, PostQuitMessage,
    SetForegroundWindow, TrackPopupMenu, HICON, HMENU, MF_STRING, TPM_NONOTIFY, TPM_RETURNCMD,
    TRACK_POPUP_MENU_FLAGS, WM_CREATE, WM_DESTROY, WM_USER,
};

use crate::shell::messenger::{
    HostToIntegrationMessage, IntegrationMessenger, IntegrationToHostMessage,
    InternalIntegrationToHostMessage,
};
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

/// GUID of the notification icon.
///
/// {0D4D901A-D36F-439A-932A-B4BABA5B104D}
const NOTIFICATION_ICON_GUID: GUID = GUID::from_values(
    0xd4d901a,
    0xd36f,
    0x439a,
    [0x93, 0x2a, 0xb4, 0xba, 0xba, 0x5b, 0x10, 0x4d],
);

/// Window message sent by the Snowland host thread to notify that a message is in the messenger.
pub const WM_SNOWLAND_MESSENGER: u32 = WM_USER + 1;

/// Window message sent by the Snowland notification system tay.
const WM_SNOWLAND_NOTIFICATION: u32 = WM_USER + 2;

/// Menu item which should quit the application.
const MENU_ITEM_EXIT: usize = 0x1;

impl ShellIntegration {
    /// Creates the shell integration.
    ///
    /// At this point the window has been fully created.
    pub fn new(messenger: IntegrationMessenger, window: HWND) -> Result<Self, Error> {
        messenger.window_created(window);

        let menu = unsafe {
            let menu = CreatePopupMenu();

            if menu.0 == 0 {
                return Err(Error::MenuCreationFailed(WinApiError::last()));
            }

            AppendMenuA(menu, MF_STRING, MENU_ITEM_EXIT, "Exit");

            menu
        };

        let icon_data = NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
            hWnd: window,
            uCallbackMessage: WM_SNOWLAND_NOTIFICATION,

            // Show the icon and tooltip and use a GUID to identify the icon calling the window
            // with a window message.
            uFlags: NIF_ICON | NIF_TIP | NIF_SHOWTIP | NIF_GUID | NIF_MESSAGE,
            hIcon: Self::load_icon(1),
            szTip: Self::make_string("Snowland"),
            guidItem: NOTIFICATION_ICON_GUID,

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
                .process_host_message(*unsafe { Box::from_raw(l_param.0 as _) })
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

            if click_result == MENU_ITEM_EXIT {
                log::info!("User requested application exit using popup menu!");
                self.messenger.send(IntegrationToHostMessage::StopRendering);
            }
        }

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
