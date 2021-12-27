use crate::graphics::XLibDisplay;
use crate::util::{Library, LibraryLoaderError};
use thiserror::Error;
use x11::xlib::{Display, XFree, XSync};

type GlXQueryVersion = unsafe extern "C" fn(*mut Display, *mut i32, *mut i32) -> bool;
type GlXChooseFBConfig =
    unsafe extern "C" fn(*mut Display, i32, *const i32, *mut i32) -> *mut std::ffi::c_void;

#[derive(Debug)]
struct GLXFunctions {
    _library: Library,
    query_version: GlXQueryVersion,
    choose_fb_config: GlXChooseFBConfig,
}

impl GLXFunctions {
    pub fn load() -> Result<Self, GLXError> {
        let library = Library::load("libGL.so").map_err(GLXError::LibraryLoadFailed)?;

        macro_rules! lookup {
            ($name:literal) => {
                unsafe { library.lookup_function_symbol($name) }.map_err(|err| {
                    GLXError::MissingRequiredFunction {
                        error: err,
                        function: $name.into(),
                    }
                })
            };
        }

        let query_version = lookup!("glXQueryVersion")?;
        let choose_fb_config = lookup!("glXChooseFBConfig")?;

        Ok(Self {
            _library: library,
            query_version,
            choose_fb_config,
        })
    }

    pub unsafe fn query_version(&self, display: *mut Display) -> (i32, i32) {
        let mut major = 0;
        let mut minor = 0;

        (self.query_version)(display, &mut major, &mut minor);

        (major, minor)
    }

    pub unsafe fn choose_framebuffer_config(
        &self,
        display: *mut Display,
        screen: i32,
        attribs: Option<&[i32]>,
    ) -> Option<(*mut std::ffi::c_void, usize)> {
        let attribs = attribs
            .map(|attribs| {
                let mut attribs = attribs.to_vec();
                attribs.push(0);

                attribs
            })
            .unwrap_or_else(|| vec![0]);

        let mut config_count = 0;
        let handle = (self.choose_fb_config)(display, screen, attribs.as_ptr(), &mut config_count);

        if handle.is_null() {
            None
        } else {
            Some((handle, config_count as _))
        }
    }
}

#[derive(Debug)]
pub struct GLXFBConfigArray<'a> {
    count: usize,
    handle: *mut std::ffi::c_void,
    display: &'a XLibDisplay,
}

impl<'a> GLXFBConfigArray<'a> {
    pub unsafe fn wrap(
        count: usize,
        handle: *mut std::ffi::c_void,
        display: &'a XLibDisplay,
    ) -> Self {
        Self {
            count,
            handle,
            display,
        }
    }

    pub fn handle(&self) -> *mut std::ffi::c_void {
        self.handle
    }
}

impl<'a> Drop for GLXFBConfigArray<'a> {
    fn drop(&mut self) {
        unsafe { XFree(self.handle) };
        self.display.sync(false);
    }
}

#[derive(Debug)]
pub struct GLX<'a> {
    display: &'a XLibDisplay,
    functions: GLXFunctions,
}

impl<'a> GLX<'a> {
    pub fn create(display: &'a XLibDisplay) -> Result<Self, GLXError> {
        let functions = GLXFunctions::load()?;

        Ok(Self { display, functions })
    }

    pub fn get_version(&self) -> (i32, i32) {
        unsafe { self.functions.query_version(self.display.handle()) }
    }

    pub fn choose_framebuffer_config(&self) -> Result<GLXFBConfigArray<'a>, GLXError> {
        let config = unsafe {
            self.functions.choose_framebuffer_config(
                self.display.handle(),
                self.display.default_screen(),
                None,
            )
        };

        Err(GLXError::NoFramebufferConfigFound)

        /* config
        .map(|(handle, count)| unsafe { GLXFBConfigArray::wrap(count, handle, self.display) })
        .ok_or(GLXError::NoFramebufferConfigFound) */
    }
}

#[derive(Debug, Error)]
pub enum GLXError {
    #[error("failed to load GL library: {0}")]
    LibraryLoadFailed(LibraryLoaderError),

    #[error("missing required function {function}: {error}")]
    MissingRequiredFunction {
        function: String,
        #[source]
        error: LibraryLoaderError,
    },

    #[error("the GLX extension is not present on the X server")]
    ExtensionNotPresent,

    #[error("no framebuffer configuration could be found for the requested attributes")]
    NoFramebufferConfigFound,
}
