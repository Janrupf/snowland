use crate::{xlib_sys, XDisplay};
use std::ffi::CString;
use std::fmt::{Debug, Formatter};

/// An X11 atom.
#[derive(Copy, Clone)]
pub struct XAtom<'a> {
    handle: xlib_sys::Atom,
    display: Option<&'a XDisplay>,
}

impl<'a> PartialEq for XAtom<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<'a> Eq for XAtom<'a> {}

impl<'a> Debug for XAtom<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        struct AtomHandleDebug<'a>(&'a XAtom<'a>);

        impl<'a> Debug for AtomHandleDebug<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} ({})", self.0.handle, self.0.name())
            }
        }

        f.debug_struct("XAtom")
            .field("handle", &AtomHandleDebug(self))
            .field("display", &self.display)
            .finish()
    }
}

impl<'a> XAtom<'a> {
    /// Wraps an existing X11 atom.
    ///
    /// # Arguments
    ///
    /// * `handle` - The underlying native X11 atom id
    /// * `display` - The display the X11 atom belongs to
    ///
    /// # Safety
    ///
    /// It is up the caller to make sure the passed handle is a valid X11 atom.
    pub unsafe fn new(handle: xlib_sys::Atom, display: &'a XDisplay) -> Self {
        Self {
            handle,
            display: Some(display),
        }
    }

    /// Retrieves the name of this X11 atom.
    pub fn name(&self) -> String {
        if let Some(display) = self.display {
            unsafe {
                let raw_name = xlib_sys::XGetAtomName(display.handle(), self.handle);
                let name = CString::from_raw(raw_name).to_string_lossy().into();

                // xlib_sys::XFree(raw_name as _);

                name
            }
        } else {
            "<unknown>".into()
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
            display: None,
        }
    }

    /// The X11 `AnyPropertyType` constant as an atom.
    pub const ANY_PROPERTY_TYPE: Self = Self::standard(xlib_sys::AnyPropertyType as _);

    /// The X11 `XA_PIXMAP` atom.
    pub const PIXMAP: Self = Self::standard(xlib_sys::XA_PIXMAP);
}
