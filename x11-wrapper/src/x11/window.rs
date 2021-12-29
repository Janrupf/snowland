use crate::{XAtom, XDisplay, XDrawable, XGeometry, XPixmap, XScreen, XVisual, XVisualInfo, XGC};
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use x11::xlib as xlib_sys;
use x11::xlib::{
    Drawable, XBlackPixelOfScreen, XClearWindow, XCreateGC, XSetBackground, XSetForeground,
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum WindowPropertyDataFormat {
    Bit8,
    Bit16,
    Bit32,
}

impl WindowPropertyDataFormat {
    pub fn from_native(format: i32) -> Option<Self> {
        match format {
            1 => Some(WindowPropertyDataFormat::Bit8),
            16 => Some(WindowPropertyDataFormat::Bit16),
            32 => Some(WindowPropertyDataFormat::Bit32),
            _ => None,
        }
    }

    pub fn to_native(&self) -> i32 {
        match self {
            WindowPropertyDataFormat::Bit8 => 8,
            WindowPropertyDataFormat::Bit16 => 16,
            WindowPropertyDataFormat::Bit32 => 32,
        }
    }

    pub fn byte_count(&self) -> usize {
        match self {
            WindowPropertyDataFormat::Bit8 => 1,
            WindowPropertyDataFormat::Bit16 => 2,
            WindowPropertyDataFormat::Bit32 => 4,
        }
    }

    pub fn byte_count_array(&self, length: usize) -> usize {
        self.byte_count() * length
    }
}

#[derive(Debug)]
pub struct WindowPropertyData<'a> {
    format: WindowPropertyDataFormat,
    actual_type: XAtom<'a>,
    item_count: usize,
    data: *mut u8,
}

impl<'a> WindowPropertyData<'a> {
    pub unsafe fn new(
        format: WindowPropertyDataFormat,
        actual_type: XAtom<'a>,
        item_count: usize,
        data: *mut u8,
    ) -> Self {
        Self {
            format,
            actual_type,
            item_count,
            data,
        }
    }

    pub fn format(&self) -> WindowPropertyDataFormat {
        self.format
    }

    pub fn ty(&self) -> XAtom<'a> {
        self.actual_type
    }

    pub fn length(&self) -> usize {
        self.item_count
    }

    pub fn byte_size(&self) -> usize {
        self.format.byte_count_array(self.item_count)
    }

    pub unsafe fn get_as_ptr<T>(&self) -> *const T {
        assert!(self.byte_size() >= std::mem::size_of::<T>());

        self.data as _
    }

    pub unsafe fn get_as_mut_ptr<T>(&self) -> *mut T {
        assert!(self.byte_size() >= std::mem::size_of::<T>());

        self.data as _
    }

    pub unsafe fn get_as_ref<T>(&self) -> &T {
        &*self.get_as_ptr::<T>()
    }

    pub unsafe fn get_as_mut_ref<T>(&mut self) -> &mut T {
        &mut *self.get_as_mut_ptr::<T>()
    }
}

impl<'a> Drop for WindowPropertyData<'a> {
    fn drop(&mut self) {
        unsafe { xlib_sys::XFree(self.data as _) };
    }
}

#[derive(Debug)]
pub enum WindowPropertyChangeMode {
    Replace,
    Prepend,
    Append,
}

impl WindowPropertyChangeMode {
    pub fn to_native(&self) -> i32 {
        match self {
            WindowPropertyChangeMode::Replace => xlib_sys::PropModeReplace,
            WindowPropertyChangeMode::Prepend => xlib_sys::PropModePrepend,
            WindowPropertyChangeMode::Append => xlib_sys::PropModeAppend,
        }
    }
}

#[derive(Debug)]
pub struct XWindow<'a> {
    handle: xlib_sys::Window,
    display: &'a XDisplay,
}

impl<'a> XWindow<'a> {
    pub unsafe fn new(handle: xlib_sys::Window, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    pub fn handle(&self) -> xlib_sys::Window {
        self.handle
    }

    pub fn get_attributes(&self) -> XWindowAttributes<'a> {
        let mut raw = MaybeUninit::uninit();
        let raw = unsafe {
            xlib_sys::XGetWindowAttributes(self.display.handle(), self.handle, raw.as_mut_ptr());

            raw.assume_init()
        };

        unsafe {
            let screen = XScreen::new(raw.screen, self.display);
            let visual = XVisual::new(raw.visual);

            XWindowAttributes::new(raw, screen, visual)
        }
    }

    pub fn clear(&self) {
        unsafe { XClearWindow(self.display.handle(), self.handle) };
    }

    pub fn get_property(
        &self,
        property: XAtom,
        offset: i64,
        length: i64,
        delete: bool,
        ty: XAtom,
    ) -> Option<(WindowPropertyData, usize)> {
        let mut actual_type = 0;
        let mut actual_format = 0;
        let mut item_count = 0;
        let mut remaining_bytes = 0;
        let mut data = std::ptr::null_mut();

        let delete = if delete { 1 } else { 0 };

        unsafe {
            xlib_sys::XGetWindowProperty(
                self.display.handle(),
                self.handle,
                property.handle(),
                offset,
                length,
                delete,
                ty.handle(),
                &mut actual_type,
                &mut actual_format,
                &mut item_count,
                &mut remaining_bytes,
                &mut data,
            )
        };

        WindowPropertyDataFormat::from_native(actual_format).map(|format| {
            let actual_type = unsafe { XAtom::new(actual_type) };
            let data =
                unsafe { WindowPropertyData::new(format, actual_type, item_count as _, data) };

            (data, remaining_bytes as _)
        })
    }

    pub fn change_property8(
        &self,
        property: XAtom,
        ty: XAtom,
        mode: WindowPropertyChangeMode,
        data: &[u8],
    ) {
        #[allow(mutable_transmutes)]
        let data = unsafe { std::mem::transmute::<_, &mut [u8]>(data) };
        let element_count = data.len();

        unsafe {
            self.change_property(
                property,
                ty,
                WindowPropertyDataFormat::Bit8,
                mode,
                data.as_mut_ptr(),
                element_count,
            )
        };
    }

    pub fn change_property16(
        &self,
        property: XAtom,
        ty: XAtom,
        mode: WindowPropertyChangeMode,
        data: &[i16],
    ) {
        #[allow(mutable_transmutes)]
        let data = unsafe { std::mem::transmute::<_, &mut [i16]>(data) };
        let element_count = data.len();

        unsafe {
            self.change_property(
                property,
                ty,
                WindowPropertyDataFormat::Bit16,
                mode,
                data.as_mut_ptr() as _,
                element_count,
            )
        };
    }

    pub fn change_property32(
        &self,
        property: XAtom,
        ty: XAtom,
        mode: WindowPropertyChangeMode,
        data: &[i32],
    ) {
        #[allow(mutable_transmutes)]
        let data = unsafe { std::mem::transmute::<_, &mut [i32]>(data) };
        let element_count = data.len();

        unsafe {
            self.change_property(
                property,
                ty,
                WindowPropertyDataFormat::Bit32,
                mode,
                data.as_mut_ptr() as _,
                element_count,
            )
        };
    }

    pub unsafe fn change_property(
        &self,
        property: XAtom,
        ty: XAtom,
        format: WindowPropertyDataFormat,
        mode: WindowPropertyChangeMode,
        data: *mut u8,
        element_count: usize,
    ) {
        xlib_sys::XChangeProperty(
            self.display.handle(),
            self.handle,
            property.handle(),
            ty.handle(),
            format.to_native(),
            mode.to_native(),
            data,
            element_count as _,
        );
    }

    pub fn delete_property(&self, property: XAtom) {
        unsafe { xlib_sys::XDeleteProperty(self.display.handle(), self.handle, property.handle()) };
    }
}

impl<'a> XDrawable<'a> for XWindow<'a> {
    fn drawable_handle(&self) -> Drawable {
        self.handle
    }

    fn display(&self) -> &'a XDisplay {
        self.display
    }
}

#[derive(Debug)]
pub struct XWindowAttributes<'a> {
    inner: xlib_sys::XWindowAttributes,
    screen: XScreen<'a>,
    visual: XVisual<'a>,
}

impl<'a> XWindowAttributes<'a> {
    pub unsafe fn new(
        inner: xlib_sys::XWindowAttributes,
        screen: XScreen<'a>,
        visual: XVisual<'a>,
    ) -> Self {
        Self {
            inner,
            screen,
            visual,
        }
    }

    pub fn screen(&self) -> &XScreen<'a> {
        &self.screen
    }

    pub fn visual(&self) -> &XVisual<'a> {
        &self.visual
    }
}
