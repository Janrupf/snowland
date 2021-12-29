use crate::xlib_sys;
use crate::XDisplay;
use crate::{XPixmap, XGC};
use std::mem::MaybeUninit;

#[derive(Debug, Default)]
pub struct XGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub border_width: u32,
    pub depth: u32,
}

pub trait XDrawable<'a>
where
    Self: Sized,
{
    fn drawable_handle(&self) -> xlib_sys::Drawable;

    fn display(&self) -> &'a XDisplay;

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

    fn create_matching_pixmap(&'a self) -> XPixmap {
        let geometry = self.get_geometry();

        self.create_pixmap(geometry.width, geometry.height, geometry.depth)
    }

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
