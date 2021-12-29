use crate::xlib_sys;
use crate::XDisplay;
use crate::{XPixmap, XGC};
use std::mem::MaybeUninit;

/// Describes the geometry of a [`XDrawable`].
#[derive(Debug, Default)]
pub struct XGeometry {
    /// The x coordinate of the drawable, only meaningful for windows
    pub x: i32,

    /// The y coordinate of the drawable, only meaningful for windows
    pub y: i32,

    /// The width of the drawable
    pub width: u32,

    /// The height of the drawable
    pub height: u32,

    /// The width of the border of the drawable, only meaningful for windows
    pub border_width: u32,

    /// The color bit-depth of the drawable
    pub depth: u32,
}

/// Represents a drawable object in the X11 protocol.
///
/// This is usually implemented for windows and pixmap's.
pub trait XDrawable<'a>
where
    Self: Sized,
{
    /// Retrieves the underlying native X11 drawable representation.
    fn drawable_handle(&self) -> xlib_sys::Drawable;

    /// Retrieves the display this drawable belongs to.
    fn display(&self) -> &'a XDisplay;

    /// Retrieves the geometry of this drawable.
    fn get_geometry(&self) -> XGeometry {
        let mut geometry = XGeometry::default();
        let mut root = 0;

        unsafe {
            xlib_sys::XGetGeometry(
                self.display().handle(),
                self.drawable_handle(),
                &mut root,
                &mut geometry.x,
                &mut geometry.y,
                &mut geometry.width,
                &mut geometry.height,
                &mut geometry.border_width,
                &mut geometry.depth,
            );
        }

        geometry
    }

    /// Creates a new X11 graphics context for rendering to the drawable.
    fn create_gc(&'a self) -> XGC<Self> {
        let mut values = MaybeUninit::uninit();

        let gc = unsafe {
            xlib_sys::XCreateGC(
                self.display().handle(),
                self.drawable_handle(),
                0,
                values.as_mut_ptr(),
            )
        };

        unsafe { XGC::new(gc, self, self.display()) }
    }

    /// Creates a pixmap matching the width, height and depth of this drawable on the same screen
    /// as this drawable resides on.
    fn create_matching_pixmap(&'a self) -> XPixmap {
        let geometry = self.get_geometry();

        self.create_pixmap(geometry.width, geometry.height, geometry.depth)
    }

    /// Creates a pixmap on the same screen as this drawable resides on.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the drawable
    /// * `height` - The height of the drawable
    /// * `depth` - The bit-depth of the drawable
    fn create_pixmap(&'a self, width: u32, height: u32, depth: u32) -> XPixmap {
        let pixmap = unsafe {
            xlib_sys::XCreatePixmap(
                self.display().handle(),
                self.drawable_handle(),
                width,
                height,
                depth,
            )
        };

        unsafe { XPixmap::new(pixmap, self.display()) }
    }
}
