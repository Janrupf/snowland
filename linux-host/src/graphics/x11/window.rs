use crate::graphics::{XDisplay, XScreen, XVisual, XVisualInfo, XGC};
use std::fmt::Debug;
use std::mem::MaybeUninit;
use x11::xlib as xlib_sys;
use x11::xlib::{XBlackPixelOfScreen, XClearWindow, XCreateGC, XSetBackground, XSetForeground};

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

    pub fn create_gc(&self) -> XGC {
        let attributes = self.get_attributes();
        let screen = attributes.screen();

        let mut values = MaybeUninit::uninit();

        let gc = unsafe { XCreateGC(self.display.handle(), self.handle, 0, values.as_mut_ptr()) };

        unsafe {
            XSetForeground(
                self.display.handle(),
                gc,
                XBlackPixelOfScreen(screen.handle()),
            );

            XSetBackground(
                self.display.handle(),
                gc,
                XBlackPixelOfScreen(screen.handle()),
            );
        }

        unsafe { XGC::new(gc, self, self.display) }
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
