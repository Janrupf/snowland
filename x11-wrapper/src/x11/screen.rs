use crate::ext::edid::MonitorDescriptor;
use crate::{xlib_sys, xrandr_sys, XAtom};
use crate::{XDisplay, XWindow};
use std::io::Cursor;
use std::slice;

/// XRandR info about a connected monitor.
#[derive(Debug)]
pub struct XRandRMonitorInfo<'a> {
    /// The X atom representing the connection name of the monitor.
    pub connection_name: XAtom<'a>,

    /// The physical name of the monitor.
    pub monitor_name: Option<String>,

    /// The serial of the monitor.
    pub monitor_serial: Option<u32>,

    /// Whether this monitor is the primary monitor.
    pub primary: bool,

    /// Whether this monitor is currently in automatic configuration mode.
    pub automatic: bool,

    /// The output number of physical connections from this virtual monitor.
    pub output_count: i32,

    /// The x coordinate at which the monitor starts.
    pub x: i32,

    /// The y coordinate at which the monitor starts.
    pub y: i32,

    /// The width of the monitor in pixels.
    pub width: i32,

    /// The height of the monitor in pixels.
    pub height: i32,

    /// The physical width of the monitor.
    pub physical_width: i32,

    /// The physical height of the monitor.
    pub physical_height: i32,
}

/// X11 screen.
///
/// Please note that while originally screens where meant to represent different heads (monitors)
/// on an X system, they rarely do anymore. Usually all monitors are combined as one huge screen
/// and the window manager takes care of assigning application windows to monitors.
///
/// Thus you can usually expect one X11 display to have one screen!
#[derive(Debug)]
pub struct XScreen<'a> {
    handle: *mut xlib_sys::Screen,
    display: &'a XDisplay,
}

impl<'a> XScreen<'a> {
    /// Wraps a native X11 screen.
    ///
    /// # Arguments
    ///
    /// * `handle` - The native platform X11 pointer of the screen
    /// * `display` - The display the screen belongs to (and often represents entirely)
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
    pub unsafe fn new(handle: *mut xlib_sys::Screen, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    /// Retrieves the underlying platform native X11 pointer.
    pub fn handle(&self) -> *mut xlib_sys::Screen {
        self.handle
    }

    /// Retrieves the number of the screen, usually 0.
    pub fn number(&self) -> i32 {
        unsafe { xlib_sys::XScreenNumberOfScreen(self.handle) }
    }

    /// Retrieves the root window of the screen.
    ///
    /// The root window is the top level background window which spans the entire screen.
    pub fn root_window(&self) -> XWindow<'a> {
        unsafe { XWindow::new((*self.handle).root, self.display) }
    }

    /// Retrieves all monitors connected to this screen.
    pub fn get_monitors(&self) -> Vec<XRandRMonitorInfo<'a>> {
        let mut monitor_count = 0;
        let info = unsafe {
            xrandr_sys::XRRGetMonitors(
                self.display.handle(),
                (*self.handle).root,
                1,
                &mut monitor_count,
            )
        };

        if info.is_null() {
            return Vec::new();
        }

        let edid_atom = self.display.get_atom("EDID");

        let mut out = Vec::with_capacity(monitor_count as _);

        for i in 0..monitor_count {
            let info = unsafe { &*info.offset(i as _) };

            let edid = edid_atom.and_then(|edid_atom| {
                if info.noutput > 0 {
                    unsafe {
                        let mut actual_type = 0;
                        let mut actual_format = 0;
                        let mut item_count = 0;
                        let mut remaining_bytes = 0;
                        let mut data = std::ptr::null_mut();

                        xrandr_sys::XRRGetOutputProperty(
                            self.display.handle(),
                            *info.outputs,
                            edid_atom.handle(),
                            0,
                            100,
                            0,
                            0,
                            xlib_sys::AnyPropertyType as _,
                            &mut actual_type,
                            &mut actual_format,
                            &mut item_count,
                            &mut remaining_bytes,
                            &mut data,
                        );

                        let edid_data = slice::from_raw_parts(data as *const u8, item_count as _);

                        let edid = crate::ext::edid::parse(&mut Cursor::new(edid_data));

                        xlib_sys::XFree(data as _);

                        edid.ok()
                    }
                } else {
                    None
                }
            });

            let (name, serial) = match edid {
                None => (None, None),
                Some(edid) => {
                    let name = edid.descriptors.0.into_iter().find_map(|desc| {
                        if let MonitorDescriptor::MonitorName(name) = desc {
                            Some(name)
                        } else {
                            None
                        }
                    });

                    (name, Some(edid.product.serial_number))
                }
            };

            out.push(XRandRMonitorInfo {
                connection_name: unsafe { XAtom::new(info.name, self.display) },
                monitor_name: name,
                monitor_serial: serial,
                primary: info.primary != 0,
                automatic: info.automatic != 0,
                output_count: info.noutput,
                x: info.x,
                y: info.y,
                width: info.width,
                height: info.height,
                physical_width: info.mwidth,
                physical_height: info.mheight,
            })
        }

        unsafe {
            xlib_sys::XFree(info as _);
        }

        out
    }
}
