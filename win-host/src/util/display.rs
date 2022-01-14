use std::collections::HashMap;
use std::ffi::OsString;

use thiserror::Error;
use windows::core::HRESULT;
use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_TARGET_DEVICE_NAME,
};
use windows::Win32::Foundation::{BOOL, ERROR_SUCCESS, LPARAM, RECT, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
    QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, MONITORINFOF_PRIMARY, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use snowland_core::rendering::display::Display;

use crate::util::extensions::FromWideNull;
use crate::WinApiError;

type EnumDisplayMonitorsUserData<'a> = (&'a mut Vec<Display>, &'a HashMap<OsString, MonitorData>);

#[derive(Debug)]
struct MonitorData {
    pub friendly_name: OsString,
    pub unique_path: OsString,
}

impl MonitorData {
    pub fn resolve(&self, id: usize, primary: bool) -> ResolvedMonitorData {
        ResolvedMonitorData {
            primary,
            friendly_name: format!("{}: {}", id, self.friendly_name.to_string_lossy()),
            unique_path: self.unique_path.to_string_lossy().into(),
        }
    }
}

#[derive(Debug)]
struct ResolvedMonitorData {
    pub primary: bool,
    pub friendly_name: String,
    pub unique_path: String,
}

impl ResolvedMonitorData {
    pub fn faked(id: usize) -> Self {
        let fake_data = || format!("Monitor {}", id);

        ResolvedMonitorData {
            primary: false,
            friendly_name: fake_data(),
            unique_path: fake_data(),
        }
    }

    pub fn into_display(self, rect: &RECT) -> Display {
        let Self {
            primary,
            friendly_name,
            unique_path,
        } = self;

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        Display::new(
            friendly_name,
            unique_path,
            primary,
            rect.left,
            rect.top,
            width,
            height,
        )
    }
}

/// Retrieves information about all active monitors for the current session.
pub fn get_displays() -> Vec<Display> {
    log::debug!("Retrieving path to name mapping...");

    // Start by mapping displays to friendly names
    let path_name_map = map_device_path_to_name().unwrap_or_else(|err| {
        log::error!(
            "Failed to retrieve display configs, name mapping monitors will fail: {}",
            err
        );
        HashMap::new()
    });

    log::debug!("Path to name mapping is: {:#?}", path_name_map);
    log::debug!("Retrieving display list...");

    // Prepare data to pass to the enumeration callback.
    let mut displays = Vec::<Display>::new();
    let mut user_data = (&mut displays, &path_name_map);

    let ok = unsafe {
        // Enumerate all active monitors for this session.
        EnumDisplayMonitors(
            None,
            std::ptr::null(),
            Some(monitor_callback),
            LPARAM(&mut user_data as *const _ as _),
        )
    }
    .as_bool();

    if !ok {
        let err = WinApiError::from_win32();
        log::error!("Failed to enumerate display monitors: {}", err);
    }

    log::info!("Found {} displays", displays.len());
    log::debug!("Found displays: {:?}", displays);

    // We now need to possibly correct the display coordinates.
    //
    // Snowland expects all coordinates to have the upper left corner at (0, 0). However, Windows
    // monitor coordinates may be negative if the monitor is left or below of the primary monitor.
    //
    // The virtual screen is a virtual rectangle which spans tightly around all monitors and
    // determines where the coordinates start. Using this information we can make the upper left
    // corner of the virtual screen equal to (0, 0) in window coordinates.
    let virtual_screen_x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let virtual_screen_y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };

    log::debug!(
        "Virtual screen top left corner is at ({}, {})",
        virtual_screen_x,
        virtual_screen_y
    );

    let correction_x = -virtual_screen_x;
    let correction_y = -virtual_screen_y;

    displays
        .into_iter()
        .map(|d| {
            let display_x = d.x();
            let display_y = d.y();

            let corrected_x = display_x + correction_x;
            let corrected_y = display_y + correction_y;

            log::debug!(
                "Moving display {} from ({}, {}) to ({}, {})",
                d.name(),
                display_x,
                display_y,
                corrected_x,
                corrected_y
            );

            d.remap_coordinates(corrected_x, corrected_y)
        })
        .collect()
}

extern "system" fn monitor_callback(
    monitor: HMONITOR,
    _hdc: HDC,
    rect: *mut RECT,
    displays: LPARAM,
) -> BOOL {
    let rect = unsafe { &*rect };
    let (displays, path_name_map) =
        unsafe { std::mem::transmute::<_, &mut EnumDisplayMonitorsUserData<'_>>(displays.0) };

    // We use this fake id in case something goes wrong and we can't query Windows for required data.
    //
    // This fake id can be used as a fallback to at least temporarily identify monitors (it will
    // probably work as long as the display setup doesn't change, but there is no guarantee
    // whatsoever as it is simple the index of the monitor array Windows gives us!)
    let fake_id = displays.len() + 1;

    let data = monitor_data_from_hmonitor(monitor, path_name_map, fake_id).unwrap_or_else(|err| {
        log::warn!(
            "Failed to retrieve name for monitor 0x{:X}: {}",
            monitor.0,
            err
        );

        ResolvedMonitorData::faked(fake_id)
    });

    // At this point name and id are either retrieved from Windows or have been filled with fake
    // data which makes the configuration at least temporary usable.
    displays.push(data.into_display(rect));

    true.into()
}

/// This function extracts as much data as possible from the HMONITOR and attempts to retrieve
/// its human friendly name as well as a unique id.
fn monitor_data_from_hmonitor(
    monitor: HMONITOR,
    path_name_map: &HashMap<OsString, MonitorData>,
    id: usize,
) -> Result<ResolvedMonitorData, MonitorDataError> {
    let mut info = MONITORINFOEXW {
        __AnonymousBase_winuser_L13571_C43: MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as _,
            ..Default::default()
        },
        ..Default::default()
    };

    // Get some basic information about the HMONITOR, we will heavily rely on that!
    if !unsafe { GetMonitorInfoW(monitor, std::mem::transmute(&mut info)) }.as_bool() {
        return Err(WinApiError::from_win32().into());
    }

    // The device path will be something like \\?\DISPLAY0
    let path = OsString::from_wide_null(&info.szDevice);

    let primary = (info.__AnonymousBase_winuser_L13571_C43.dwFlags & MONITORINFOF_PRIMARY) != 0;

    // We can't get the friendly name from the HMONITOR itself, but earlier on we queried the
    // hardware for that information, so we now use our GDI path to map to that name as well as
    // unique id (or well, path).
    path_name_map
        .get(&path)
        .map(|data| data.resolve(id, primary))
        .ok_or(MonitorDataError::DataNotFound(path))
}

#[derive(Debug, Error)]
enum MonitorDataError {
    #[error(transparent)]
    WinApi(#[from] WinApiError),

    #[error("data for monitor {} not found", .0.to_string_lossy())]
    DataNotFound(OsString),
}

fn map_device_path_to_name() -> Result<HashMap<OsString, MonitorData>, WinApiError> {
    let mut path_num = 0;
    let mut mode_num = 0;

    // Retrieve the amount of active display configurations.
    //
    // We will later use them to retrieve both the device path as well as the friendly name.
    let res = WIN32_ERROR::from(unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_num, &mut mode_num)
    } as u32);

    if res != ERROR_SUCCESS {
        return Err(WinApiError::from(HRESULT(res.0)));
    }

    // Make space so that all active configurations can be stored.
    let mut paths = Vec::with_capacity(path_num as _);
    let mut modes = Vec::with_capacity(mode_num as _);

    // Ask the system to provide the active configurations
    let res = WIN32_ERROR::from(unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_num,
            paths.as_mut_ptr(),
            &mut mode_num,
            modes.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    } as u32);

    if res != ERROR_SUCCESS {
        return Err(WinApiError::from(HRESULT(res.0)));
    }

    // Change the vector lengths to match the amount of configurations.
    unsafe {
        paths.set_len(path_num as _);
        modes.set_len(mode_num as _);
    }

    let mut mapping = HashMap::new();

    // Start mapping all active configurations.
    for p in paths {
        // We first need to get the source.
        //
        // The source is what contains the GDI compatible path which we use later
        // to map from HMONITOR. HMONITORs refer to GDI paths and can be seen as
        // virtual (software) side of the display connection.
        let mut source_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                size: std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as _,
                adapterId: p.sourceInfo.adapterId,
                id: p.sourceInfo.id,
            },
            ..Default::default()
        };

        let res = WIN32_ERROR::from(unsafe {
            DisplayConfigGetDeviceInfo(std::mem::transmute(&mut source_name))
        } as u32);

        if res != ERROR_SUCCESS {
            let err = WinApiError::from(HRESULT(res.0));

            log::warn!(
                "Skipping display path {}@{:?} as the source info could not be retrieved: {}",
                p.sourceInfo.id,
                p.sourceInfo.adapterId,
                err
            );

            continue;
        }

        // Now we retrieve the target.
        //
        // The target in this case represents the physical monitor (or well, the physical
        // endpoint). In this case we use it to query the hardware for it's name (thus
        // retrieving the real name of the monitor directly from it).
        let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
            header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as _,
                adapterId: p.targetInfo.adapterId,
                id: p.targetInfo.id,
            },
            ..Default::default()
        };

        let res = WIN32_ERROR::from(unsafe {
            DisplayConfigGetDeviceInfo(std::mem::transmute(&mut target_name))
        } as u32);

        if res != ERROR_SUCCESS {
            let err = WinApiError::from(HRESULT(res.0));

            log::warn!(
                "Skipping display path {}@{:?} as the target info could not be retrieved: {}",
                p.targetInfo.id,
                p.targetInfo.adapterId,
                err
            );

            continue;
        }

        // Device path at this point is something like \\?\DISPLAY0
        let device_path = OsString::from_wide_null(&source_name.viewGdiDeviceName);

        // Friendly name at this point (hopefully...) is a human readable name of the monitor
        let friendly_name = OsString::from_wide_null(&target_name.monitorFriendlyDeviceName);

        // This should be a unique path for the monitor, something like
        // \\?\DISPLAY#SOME_NAME#SOME_ID#SOME_UUID
        let unique_path = OsString::from_wide_null(&target_name.monitorDevicePath);

        mapping.insert(
            device_path,
            MonitorData {
                friendly_name,
                unique_path,
            },
        );
    }

    Ok(mapping)
}
