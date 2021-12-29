//! This library is an X11 Rust wrapper which provides a mostly safe interface for XLib.
//!
//! Please note that the available functionality has been tailored to fit Snowland and as such
//! this wrapper does not reflect all of X11. A lot of error checks are also missing (due to X11
//! bad error handling mechanism).

mod glx;
mod x11;

pub use self::x11::*;
pub use glx::*;

pub use ::x11::glx as glx_sys;
pub use ::x11::glx::arb as glx_arb_sys;
pub use ::x11::xlib as xlib_sys;
