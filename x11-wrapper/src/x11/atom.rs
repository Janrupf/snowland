use std::marker::PhantomData;
use x11::xlib as xlib_sys;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct XAtom<'a> {
    handle: xlib_sys::Atom,
    _data: PhantomData<&'a ()>,
}

impl<'a> XAtom<'a> {
    pub unsafe fn new(handle: xlib_sys::Atom) -> Self {
        Self {
            handle,
            _data: PhantomData,
        }
    }

    pub fn handle(&self) -> xlib_sys::Atom {
        self.handle
    }
}

impl XAtom<'static> {
    const fn standard(handle: xlib_sys::Atom) -> Self {
        Self {
            handle,
            _data: PhantomData,
        }
    }

    pub const ANY_PROPERTY_TYPE: Self = Self::standard(xlib_sys::AnyPropertyType as _);
    pub const PIXMAP: Self = Self::standard(xlib_sys::XA_PIXMAP);
}
