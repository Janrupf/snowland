mod context;
mod fb_config;
mod pixmap;

pub use context::*;
pub use pixmap::*;

pub use fb_config::*;
use std::ffi::{CStr, CString};

use crate::glx_arb_sys;
use crate::glx_sys;
use crate::xlib_sys;

use crate::{XDisplay, XScreen, XVisual};
use crate::{XDrawable, XPixmap};
use thiserror::Error;

type GLXCreateContextAttribsARBFn = unsafe extern "C" fn(
    *mut xlib_sys::Display,
    glx_sys::GLXFBConfig,
    glx_sys::GLXContext,
    i32,
    *const i32,
) -> glx_sys::GLXContext;

const ARB_CREATE_CONTEXT_EXTENSION: &str = "GLX_ARB_create_context";

#[derive(Debug)]
pub struct GLX<'a> {
    display: &'a XDisplay,
}

impl<'a> GLX<'a> {
    pub fn create(display: &'a XDisplay) -> Result<Self, GLXError> {
        Ok(Self { display })
    }

    pub fn get_version(&self) -> (i32, i32) {
        let mut major = 0;
        let mut minor = 0;

        unsafe { glx_sys::glXQueryVersion(self.display.handle(), &mut major, &mut minor) };

        (major, minor)
    }

    pub fn lookup_function(&self, name: impl AsRef<str>) -> Option<unsafe extern "C" fn()> {
        let name = match CString::new(name.as_ref()) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let name = name.as_bytes_with_nul().as_ptr();

        unsafe { glx_sys::glXGetProcAddressARB(name).or_else(|| glx_sys::glXGetProcAddress(name)) }
    }

    pub fn find_framebuffer_config(
        &self,
        screen: &XScreen,
        visual: &XVisual,
    ) -> Result<GLXFBConfig, GLXError> {
        let configs = self.retrieve_framebuffer_configs(screen)?;

        let mut chosen_config = None;

        for config in configs.iter() {
            let select = matches!(config.get_visual(), Some(config_visual) if config_visual.visual().id() == visual.id());

            if select {
                chosen_config = Some(config.extend_lifetime(self.display));
                break;
            }
        }

        chosen_config.ok_or(GLXError::NoFramebufferConfigFound)
    }

    pub fn convert_pixmap(&self, config: &GLXFBConfig, x_pixmap: XPixmap<'a>) -> GLXPixmap<'a> {
        let pixmap = unsafe {
            glx_sys::glXCreateGLXPixmap(
                self.display.handle(),
                config.get_visual().unwrap().handle(),
                x_pixmap.drawable_handle(),
            )
        };

        unsafe { GLXPixmap::new(pixmap, x_pixmap, self.display) }
    }

    pub fn create_context(
        &self,
        screen: &XScreen,
        config: &GLXFBConfig,
    ) -> Result<GLXContext, GLXError> {
        let extensions = self.query_extensions(screen);

        let glx_create_context_attribs_arb = unsafe {
            glx_sys::glXGetProcAddressARB(
                CString::new("glXCreateContextAttribsARB")
                    .unwrap()
                    .to_bytes()
                    .as_ptr(),
            )
        };

        let arb_context_supported = extensions.contains(&ARB_CREATE_CONTEXT_EXTENSION);

        let glx_context = match (glx_create_context_attribs_arb, arb_context_supported) {
            (Some(glx_create_context_attribs_arb), true) if false => {
                #[rustfmt::skip]
                    let context_attribs = &[
                    glx_arb_sys::GLX_CONTEXT_MAJOR_VERSION_ARB, 3,
                    glx_arb_sys::GLX_CONTEXT_MINOR_VERSION_ARB, 3,
                    0
                ];

                let glx_create_context_attribs_arb = unsafe {
                    std::mem::transmute::<_, GLXCreateContextAttribsARBFn>(
                        glx_create_context_attribs_arb,
                    )
                };

                unsafe {
                    glx_create_context_attribs_arb(
                        self.display.handle(),
                        config.handle(),
                        std::ptr::null_mut(),
                        1,
                        context_attribs.as_ptr(),
                    )
                }
            }
            (_, _) => unsafe {
                glx_sys::glXCreateNewContext(
                    self.display.handle(),
                    config.handle(),
                    glx_sys::GLX_RGBA_TYPE,
                    std::ptr::null_mut(),
                    1,
                )
            },
        };

        Ok(unsafe { GLXContext::new(glx_context, self.display) })
    }

    pub fn query_extensions(&self, screen: &XScreen) -> Vec<&'static str> {
        let all: &'static CStr = unsafe {
            let ptr = glx_sys::glXQueryExtensionsString(self.display.handle(), screen.number());

            CStr::from_ptr(ptr)
        };

        all.to_str()
            .expect("OpenGL extensions should never include non Unicode chars")
            .split(' ')
            .collect()
    }

    fn retrieve_framebuffer_configs(
        &self,
        screen: &XScreen,
    ) -> Result<GLXFBConfigArray<'a>, GLXError> {
        let mut config_count = 0;

        let configs = unsafe {
            glx_sys::glXChooseFBConfig(
                self.display.handle(),
                screen.number(),
                [0].as_ptr(),
                &mut config_count,
            )
        };

        if configs.is_null() {
            return Err(GLXError::NoFramebufferConfigFound);
        }

        Ok(unsafe { GLXFBConfigArray::wrap(config_count as _, configs, self.display) })
    }
}

#[derive(Debug, Error)]
pub enum GLXError {
    #[error("the GLX extension is not present on the X server")]
    ExtensionNotPresent,

    #[error("no framebuffer configuration could be found for the requested attributes")]
    NoFramebufferConfigFound,

    #[error("0x{0:X} is not a valid GLX FBConfig attribute")]
    BadAttribute(i32),

    #[error("GLX call failed with error 0x{0:X}")]
    GenericError(i32),
}
