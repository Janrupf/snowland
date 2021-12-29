use crate::xlib_sys;
use std::marker::PhantomData;

/// An X11 atom.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct XAtom<'a> {
    handle: xlib_sys::Atom,
    _data: PhantomData<&'a ()>,
}

impl<'a> XAtom<'a> {
    /// Wraps an existing X11 atom.
    ///
    /// # Arguments
    ///
    /// * `handle` - The underlying native X11 atom id
    ///
    /// # Safety
    ///
    /// It is up the caller to make sure the passed handle is a valid X11 atom.
    pub unsafe fn new(handle: xlib_sys::Atom) -> Self {
        Self {
            handle,
            _data: PhantomData,
        }
    }

    /// Retrieves the underlying X11 atom id.
    pub fn handle(&self) -> xlib_sys::Atom {
        self.handle
    }
}

impl XAtom<'static> {
    /// Creates a new atom from a standard X11 definition.
    const fn standard(handle: xlib_sys::Atom) -> Self {
        Self {
            handle,
            _data: PhantomData,
        }
    }

    /// The X11 `AnyPropertyType` constant as an atom.
    pub const ANY_PROPERTY_TYPE: Self = Self::standard(xlib_sys::AnyPropertyType as _);

    /// The X11 `XA_PIXMAP` atom.
    pub const PIXMAP: Self = Self::standard(xlib_sys::XA_PIXMAP);
}
