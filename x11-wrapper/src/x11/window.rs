use crate::xlib_sys;
use crate::{XAtom, XDisplay, XDrawable, XGeometry, XPixmap, XScreen, XVisual, XVisualInfo, XGC};
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use x11::xlib::{
    Drawable, XBlackPixelOfScreen, XClearWindow, XCreateGC, XSetBackground, XSetForeground,
};

/// Describes the possible format of a X11 window property.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum WindowPropertyDataFormat {
    /// One property element is 8 bits long
    Bit8,

    /// One property element is 16 bits long
    Bit16,

    /// One property element is 32 bits long
    Bit32,
}

impl WindowPropertyDataFormat {
    /// Attempts to convert the format from the X11 native representation.
    ///
    /// # Arguments
    ///
    /// * `format` - The native format, must be one of 8, 16 or 32
    pub fn from_native(format: i32) -> Option<Self> {
        match format {
            8 => Some(WindowPropertyDataFormat::Bit8),
            16 => Some(WindowPropertyDataFormat::Bit16),
            32 => Some(WindowPropertyDataFormat::Bit32),
            _ => None,
        }
    }

    /// Converts this format to the native representation.
    pub fn to_native(&self) -> i32 {
        match self {
            WindowPropertyDataFormat::Bit8 => 8,
            WindowPropertyDataFormat::Bit16 => 16,
            WindowPropertyDataFormat::Bit32 => 32,
        }
    }

    /// Returns the amount of bytes per property.
    pub fn byte_count(&self) -> usize {
        match self {
            WindowPropertyDataFormat::Bit8 => 1,
            WindowPropertyDataFormat::Bit16 => 2,
            WindowPropertyDataFormat::Bit32 => 4,
        }
    }

    /// Returns the amount of bytes for a property array.
    ///
    /// # Arguments
    ///
    /// * `length` - The length of the array
    pub fn byte_count_array(&self, length: usize) -> usize {
        self.byte_count() * length
    }
}

/// Represents data held by a window property.
#[derive(Debug)]
pub struct WindowPropertyData<'a> {
    format: WindowPropertyDataFormat,
    actual_type: XAtom<'a>,
    item_count: usize,
    data: *mut u8,
}

impl<'a> WindowPropertyData<'a> {
    /// Wraps native window property data.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the data
    /// * `actual_type` - The actual type of the data as reported by the X server
    /// * `item_count` - The amount of properties stored in the data
    /// * `data` - A pointer to the beginning of the stored data
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
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

    /// Retrieves the format of the property elements.
    pub fn format(&self) -> WindowPropertyDataFormat {
        self.format
    }

    /// Retrieves the type of the property elements as reported by the X server.
    pub fn ty(&self) -> XAtom<'a> {
        self.actual_type
    }

    /// Retrieves the amount of properties in the data.
    pub fn length(&self) -> usize {
        self.item_count
    }

    /// Retrieves the size of the entire data in bytes.
    pub fn byte_size(&self) -> usize {
        self.format.byte_count_array(self.item_count)
    }

    /// Interprets the data as a pointer of a specific type.
    ///
    /// # Panics
    ///
    /// If the size of the stored data is smaller than the size of the requested type.
    pub fn get_as_ptr<T>(&self) -> *const T {
        assert!(self.byte_size() < std::mem::size_of::<T>());

        self.data as _
    }

    /// Interprets the data as a mutable pointer of a specific type.
    ///
    /// # Panics
    ///
    /// If the size of the stored data is smaller than the size of the requested type.
    pub fn get_as_mut_ptr<T>(&self) -> *mut T {
        assert!(self.byte_size() < std::mem::size_of::<T>());

        self.data as _
    }

    /// Interprets the data as a reference of a specific type.
    ///
    /// # Panics
    ///
    /// If the size of the stored data is smaller than the size of the requested type.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the underlying data is valid for the requested type.
    pub unsafe fn get_as_ref<T>(&self) -> &T {
        &*self.get_as_ptr::<T>()
    }

    /// Interprets the data as a mutable reference of a specific type.
    ///
    /// # Panics
    ///
    /// If the size of the stored data is smaller than the size of the requested type.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that the underlying data is valid for the requested type.
    pub unsafe fn get_as_mut_ref<T>(&mut self) -> &mut T {
        &mut *self.get_as_mut_ptr::<T>()
    }
}

impl<'a> Drop for WindowPropertyData<'a> {
    fn drop(&mut self) {
        unsafe { xlib_sys::XFree(self.data as _) };
    }
}

/// Describes how the change of a window property is performed.
#[derive(Debug)]
pub enum WindowPropertyChangeMode {
    Replace,
    Prepend,
    Append,
}

impl WindowPropertyChangeMode {
    /// Converts the change mode the native X11 representation.
    pub fn to_native(&self) -> i32 {
        match self {
            WindowPropertyChangeMode::Replace => xlib_sys::PropModeReplace,
            WindowPropertyChangeMode::Prepend => xlib_sys::PropModePrepend,
            WindowPropertyChangeMode::Append => xlib_sys::PropModeAppend,
        }
    }
}

/// Represents a window on the X server.
#[derive(Debug)]
pub struct XWindow<'a> {
    handle: xlib_sys::Window,
    display: &'a XDisplay,
}

impl<'a> XWindow<'a> {
    /// Wraps an existing window native X11 window handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The native X11 window to wrap
    /// * `display` - The X11 display the window belongs to
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that all arguments are valid.
    pub unsafe fn new(handle: xlib_sys::Window, display: &'a XDisplay) -> Self {
        Self { handle, display }
    }

    /// Retrieves the underlying native X11 window handle.
    pub fn handle(&self) -> xlib_sys::Window {
        self.handle
    }

    /// Retrieves the attributes of the window.
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

    /// Clears the content area of the window.
    pub fn clear(&self) {
        unsafe { XClearWindow(self.display.handle(), self.handle) };
    }

    /// Attempts to retrieve a window property.
    ///
    /// This functions returns (if available) the read data and amount of remaining bytes.
    ///
    /// # Arguments
    ///
    /// * `property` - The X atom identifying the property
    /// * `offset` - The byte offset into the property to start reading at
    /// * `length` - The maximal amount of bytes to read
    /// * `delete` - Whether the property should be deleted upon retrieval
    /// * `ty` - The X atom identifying the expected type of the property
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

    /// Changes a property in 8 bit format,
    ///
    /// # Arguments
    ///
    /// * `property` - The X atom identifying the property
    /// * `ty` - The X atom identifying the property type
    /// * `mode` - How the property should be changed
    /// * `data` - The data to work with (interpretation depends on `mode`)
    pub fn change_property8(
        &self,
        property: XAtom,
        ty: XAtom,
        mode: WindowPropertyChangeMode,
        data: &[u8],
    ) {
        // XChangeProperty never writes to data, but it is not defined as const in C
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

    /// Changes a property in 16 bit format,
    ///
    /// # Arguments
    ///
    /// * `property` - The X atom identifying the property
    /// * `ty` - The X atom identifying the property type
    /// * `mode` - How the property should be changed
    /// * `data` - The data to work with (interpretation depends on `mode`)
    pub fn change_property16(
        &self,
        property: XAtom,
        ty: XAtom,
        mode: WindowPropertyChangeMode,
        data: &[i16],
    ) {
        // XChangeProperty never writes to data, but it is not defined as const in C
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

    /// Changes a property in 32 bit format,
    ///
    /// # Arguments
    ///
    /// * `property` - The X atom identifying the property
    /// * `ty` - The X atom identifying the property type
    /// * `mode` - How the property should be changed
    /// * `data` - The data to work with (interpretation depends on `mode`)
    pub fn change_property32(
        &self,
        property: XAtom,
        ty: XAtom,
        mode: WindowPropertyChangeMode,
        data: &[i32],
    ) {
        // XChangeProperty never writes to data, but it is not defined as const in C
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

    /// Changes a property,
    ///
    /// # Arguments
    ///
    /// * `property` - The X atom identifying the property
    /// * `ty` - The X atom identifying the property type
    /// * `format` - The format of the property
    /// * `mode` - How the property should be changed
    /// * `data` - The data to work with (interpretation depends on `mode`)
    /// * `element_count` - The amount of elements stored in `data`
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
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

    /// Deletes a property if it exists from the window.
    ///
    /// # Arguments
    ///
    /// * `property` - The X atom identifying the property
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

/// Properties of an X11 window.
#[derive(Debug)]
pub struct XWindowAttributes<'a> {
    inner: xlib_sys::XWindowAttributes,
    screen: XScreen<'a>,
    visual: XVisual<'a>,
}

impl<'a> XWindowAttributes<'a> {
    /// Wraps native X11 window properties.
    ///
    /// # Arguments
    ///
    /// * `inner` - The native X11 window attributes data
    /// * `screen` - The screen depicted in the window attributes
    /// * `visual` - The visual depicted in the window attributes
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure all arguments are valid.
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

    /// Retrieves the screen of the window these attributes describe.
    pub fn screen(&self) -> &XScreen<'a> {
        &self.screen
    }

    /// Retrieves the visual of the window these attributes describe
    pub fn visual(&self) -> &XVisual<'a> {
        &self.visual
    }
}
