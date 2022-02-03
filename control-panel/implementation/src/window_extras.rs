use nativeshell::shell::{ContextRef, WindowHandle};

#[cfg(windows)]
pub fn set_window_icon(context: &ContextRef, window: WindowHandle) {
    use windows::Win32::Foundation::{HWND, LPARAM, PSTR, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleA;
    use windows::Win32::UI::WindowsAndMessaging::{
        LoadIconA, SendMessageA, ICON_BIG, ICON_SMALL, WM_SETICON,
    };

    let hwnd = HWND(
        context
            .window_manager
            .borrow()
            .get_platform_window(window)
            .unwrap(),
    );

    unsafe {
        let own_handle = GetModuleHandleA(None);
        let icon = LoadIconA(own_handle, PSTR(1 as *mut _));

        SendMessageA(
            hwnd,
            WM_SETICON,
            WPARAM(ICON_SMALL as _),
            LPARAM(icon.0 as _),
        );
        SendMessageA(hwnd, WM_SETICON, WPARAM(ICON_BIG as _), LPARAM(icon.0 as _));
    }
}

#[cfg(not(windows))]
pub fn set_window_icon(context: &ContextRef, window: WindowHandle) {}
