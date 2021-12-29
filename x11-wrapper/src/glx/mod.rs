//! This module provides all sort of functionality to interact with GLX.

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

/// Type alias for the [`glXCreateContextAttribsARB`] C function.
type GLXCreateContextAttribsARBFn = unsafe extern "C" fn(
    *mut xlib_sys::Display,
    glx_sys::GLXFBConfig,
    glx_sys::GLXContext,
    i32,
    *const i32,
) -> glx_sys::GLXContext;

/// The name of the GLX extension providing ARB context creation.
const ARB_CREATE_CONTEXT_EXTENSION: &str = "GLX_ARB_create_context";

/// Main interface for talking to GLX.
///
/// This interface is only valid as long as the display is held open. However, the functions
/// used here are not loaded from the display but rather the `libGL.so` or its variations.
#[derive(Debug)]
pub struct GLX<'a> {
    display: &'a XDisplay,
}

impl<'a> GLX<'a> {
    /// Creates the GLX interface.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn open_display() {}
    /// # struct GLX;
    /// # impl GLX {
    /// #     pub fn create(display: &()) -> Result<Self, ()> { Ok(Self {}) }
    /// # }
    /// #
    /// let display = open_display();
    /// let glx = GLX::create(&display).unwrap();
    /// ```
    ///
    /// # Note
    ///
    /// For now this function can't fail, but it might in the future, thus a result is returned.
    pub fn create(display: &'a XDisplay) -> Result<Self, GLXError> {
        Ok(Self { display })
    }

    /// Retrieves the version of GLX present on the display.
    ///
    /// # Examples
    ///
    /// ```
    /// # struct GLX {}
    /// # impl GLX {
    /// #     pub fn get_version(&self) -> (i32, i32) {
    /// #         (1, 3)
    /// #     }
    /// # }
    /// #
    /// # let glx = GLX {};
    /// #
    /// let (major, minor) = glx.get_version();
    /// println!("GLX version is {}.{}", major, minor);
    /// ```
    pub fn get_version(&self) -> (i32, i32) {
        let mut major = 0;
        let mut minor = 0;

        unsafe { glx_sys::glXQueryVersion(self.display.handle(), &mut major, &mut minor) };

        (major, minor)
    }

    /// Looks up an OpenGL function from GLX.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to look up
    ///
    /// # Warning
    ///
    /// Some implementations of GLX return a function pointer for every name starting with "gl".
    /// This means that this function might actually return pointers for OpenGL functions which
    /// don't exist or are not supported!
    ///
    /// # Examples
    ///
    /// ```
    /// # struct GLX {}
    /// # impl GLX {
    /// #     pub fn lookup_function(&self, name: impl AsRef<str>) -> Option<unsafe extern "C" fn()> {
    /// #         None
    /// #     }
    /// # }
    /// #
    /// # let glx = GLX {};
    /// #
    /// let gl_clear = glx.lookup_function("glClear");
    /// println!("glClear = {:#?}", gl_clear);
    /// ```
    ///
    pub fn lookup_function(&self, name: impl AsRef<str>) -> Option<unsafe extern "C" fn()> {
        let name = match CString::new(name.as_ref()) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let name = name.as_bytes_with_nul().as_ptr();

        unsafe { glx_sys::glXGetProcAddressARB(name).or_else(|| glx_sys::glXGetProcAddress(name)) }
    }

    /// Attempts to find a framebuffer configuration matching the specified visual.
    ///
    /// This function may be used to find a framebuffer configuration which can be used on an
    /// already created visual, which might be controlled externally (for example a window which
    /// was created by another X client.
    ///
    /// # Arguments
    ///
    /// * `screen` - The X screen on which the visual resides
    /// * `visual` - The visual to look up the framebuffer configuration for
    ///
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

    /// Converts an existing X11 pixmap into a GLX pixmap.
    ///
    /// This can be used to render to X11 pixmap's using OpenGL or to use an X11 pixmap as a texture
    /// in OpenGL.
    ///
    /// It is always attempted to create an OpenGL 3.0 or newer context, however, this might fail
    /// and an older context might be returned.
    ///
    /// # Arguments
    ///
    /// * `config` - The framebuffer configuration to use for the pixmap
    /// * `x_pixmap` - The X11 pixmap to wrap
    ///
    /// # Warning
    ///
    /// Testing has revealed that this works mediocre at best, your millage may vary! On modern X11
    /// connections its a hit or miss whether this works reliably.
    ///
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

    /// Creates an GLX OpenGL context.
    ///
    /// # Arguments
    ///
    /// * `screen` - The screen to create the context on
    /// * `config` - The framebuffer configuration to use for rendering
    ///
    /// # Examples
    ///
    /// ```
    /// # use snowland_x11_wrapper::GLXError;
    /// #
    /// # struct GLX {}
    /// # impl GLX {
    /// #     pub fn create_context(&self, screen: &(), config: &()) -> Result<GLXContext, GLXError> {
    /// #         Ok(GLXContext {})
    /// #     }
    /// # }
    /// #
    /// # struct GLXContext;
    /// # impl GLXContext {
    /// #     pub fn make_current(&self, drawable: &()) {}
    /// # }
    /// #
    /// # let screen = ();
    /// # let config = ();
    /// # let glx = GLX {};
    /// # let window = ();
    /// #
    /// let context = glx.create_context(&screen, &config).expect("Failed to create GLX context");
    /// context.make_current(&window);
    ///
    /// // From here on you can use OpenGL functions.
    /// ```
    ///
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

    /// Queries all available GLX extensions.
    ///
    /// # Arguments
    ///
    /// * `screen` - The screen to query extensions for
    ///
    /// # Examples  
    /// ```
    /// # struct GLX {}
    /// # impl GLX {
    /// #     pub fn query_extensions(&self, screen: &()) -> Vec<&'static str> {
    /// #         Vec::new()
    /// #     }
    /// # }
    /// # let screen = ();
    /// # let glx = GLX {};
    /// #
    /// let extensions = glx.query_extensions(&screen);
    /// println!("There are {} extensions available: {:#?}", extensions.len(), extensions);
    /// ```
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

/// Possible errors that might occur while working with GLX.
#[derive(Debug, Error)]
pub enum GLXError {
    /// The GLX extension is missing from the X server.
    ///
    /// This usually happens when no appropriate graphics driver is installed or the connection
    /// is running over the network. When running a network connection it may be possible to use
    /// indirect GLX when enabling indirect rendering extensions on the X server.
    #[error("the GLX extension is not present on the X server")]
    ExtensionNotPresent,

    /// Attempts to find a framebuffer configuration failed.
    #[error("no framebuffer configuration could be found for the requested attributes")]
    NoFramebufferConfigFound,

    /// An attempt was made to request an invalid attribute from a GLX framebuffer configuration.
    #[error("0x{0:X} is not a valid GLX FBConfig attribute")]
    BadAttribute(i32),

    /// Some unexpected error occurred while talking to GLX.
    #[error("GLX call failed with error 0x{0:X}")]
    GenericError(i32),
}
