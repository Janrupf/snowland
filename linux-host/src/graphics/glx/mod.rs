mod context;
mod fb_config;
pub use context::*;

pub use fb_config::*;
use std::ffi::{CStr, CString, NulError};

use crate::graphics::{XDisplay, XScreen, XWindow};
use thiserror::Error;
use x11::glx as glx_sys;
use x11::glx::arb as glx_arb_sys;
use x11::xlib as xlib_sys;

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

        let name = name.as_bytes().as_ptr();

        unsafe { glx_sys::glXGetProcAddressARB(name).or_else(|| glx_sys::glXGetProcAddress(name)) }
    }

    pub fn create_context(&self, window: &XWindow) -> Result<GLXContext, GLXError> {
        let window_attributes = window.get_attributes();

        let configs = self.retrieve_framebuffer_configs(window_attributes.screen())?;

        log::debug!("Choosing out of {} framebuffer configs...", configs.count());

        let mut chosen_config = None;

        for config in configs.iter() {
            if let Some(visual) = config.get_visual() {
                if window_attributes.visual().id() == visual.visual().id() {
                    chosen_config = Some(config.clone());
                }
            }
        }

        log::debug!("Chosen FB config: {:#?}", chosen_config);

        let chosen_config = chosen_config.ok_or(GLXError::NoFramebufferConfigFound)?;

        let extensions = self.query_extensions(window_attributes.screen());
        log::debug!("Supported extensions: {:#?}", extensions);

        let glx_create_context_attribs_arb = unsafe {
            glx_sys::glXGetProcAddressARB(
                CString::new("glXCreateContextAttribsARB")
                    .unwrap()
                    .to_bytes()
                    .as_ptr(),
            )
        };

        let arb_context_supported = extensions.contains(&ARB_CREATE_CONTEXT_EXTENSION);

        log::trace!(
            "glXCreateContextAttribsARB = {:?}, arb_context_supported = {}",
            glx_create_context_attribs_arb,
            arb_context_supported
        );

        let glx_context = match (glx_create_context_attribs_arb, arb_context_supported) {
            (Some(glx_create_context_attribs_arb), true) => {
                #[rustfmt::skip]
                    let context_attribs = &[
                    glx_arb_sys::GLX_CONTEXT_MAJOR_VERSION_ARB, 3,
                    glx_arb_sys::GLX_CONTEXT_MINOR_VERSION_ARB, 3,
                    0
                ];

                log::debug!("Creating GLX context using ARB extension...");

                let glx_create_context_attribs_arb = unsafe {
                    std::mem::transmute::<_, GLXCreateContextAttribsARBFn>(
                        glx_create_context_attribs_arb,
                    )
                };

                unsafe {
                    glx_create_context_attribs_arb(
                        self.display.handle(),
                        chosen_config.handle(),
                        std::ptr::null_mut(),
                        1,
                        context_attribs.as_ptr(),
                    )
                }
            }
            (_, _) => {
                log::warn!("Falling back to old GLX context creation...");

                unsafe {
                    glx_sys::glXCreateNewContext(
                        self.display.handle(),
                        chosen_config.handle(),
                        glx_sys::GLX_RGBA_TYPE,
                        std::ptr::null_mut(),
                        1,
                    )
                }
            }
        };

        log::debug!("Created GLX context {:p}", glx_context);

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
