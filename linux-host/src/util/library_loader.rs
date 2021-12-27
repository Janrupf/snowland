use std::ffi::{CStr, CString, NulError};
use thiserror::Error;

const RTLD_LAZY: i32 = 0x00001;

extern "C" {
    fn dlopen(data: *const std::os::raw::c_char, flags: i32) -> *mut std::ffi::c_void;
    fn dlclose(data: *const std::ffi::c_void) -> i32;

    fn dlerror() -> *mut std::os::raw::c_char;
    fn dlsym(
        handle: *mut std::ffi::c_void,
        symbol: *const std::os::raw::c_char,
    ) -> *mut std::ffi::c_void;
}

#[derive(Debug, Clone, Error)]
pub enum LibraryLoaderError {
    #[error("{0}")]
    OpenFailed(String),

    #[error("invalid library name: {0}")]
    InvalidLibraryName(NulError),

    #[error("invalid symbol name: {0}")]
    InvalidSymbolName(NulError),

    #[error("{0}")]
    SymbolNotFound(String),
}

impl LibraryLoaderError {
    pub fn open_error() -> Self {
        Self::OpenFailed(Self::last_os_error())
    }

    pub fn symbol_not_found() -> Self {
        Self::SymbolNotFound(Self::last_os_error())
    }

    fn last_os_error() -> String {
        unsafe { CStr::from_ptr(dlerror()).to_string_lossy().into() }
    }
}

#[derive(Debug)]
pub struct Library {
    handle: *mut std::ffi::c_void,
}

impl Library {
    pub fn load(library_name: impl AsRef<str>) -> Result<Self, LibraryLoaderError> {
        let c_string =
            CString::new(library_name.as_ref()).map_err(LibraryLoaderError::InvalidLibraryName)?;

        let handle = unsafe { dlopen(c_string.as_ptr(), RTLD_LAZY) };
        if handle.is_null() {
            return Err(LibraryLoaderError::open_error());
        }

        Ok(Self { handle })
    }

    pub unsafe fn lookup_data_symbol<T>(
        &self,
        symbol_name: impl AsRef<str>,
    ) -> Result<&T, LibraryLoaderError> {
        let symbol = self.lookup_address(symbol_name)?;
        let value_ref = &*(symbol as *const T);

        Ok(value_ref)
    }

    pub unsafe fn lookup_function_symbol<T>(
        &self,
        symbol_name: impl AsRef<str>,
    ) -> Result<T, LibraryLoaderError>
    where
        T: Copy,
    {
        let symbol = self.lookup_address(symbol_name)?;
        let fn_ref = *std::mem::transmute::<_, &T>(&symbol);

        Ok(fn_ref)
    }

    pub fn lookup_address(
        &self,
        symbol_name: impl AsRef<str>,
    ) -> Result<*mut std::ffi::c_void, LibraryLoaderError> {
        let c_string =
            CString::new(symbol_name.as_ref()).map_err(LibraryLoaderError::InvalidSymbolName)?;

        let symbol = unsafe { dlsym(self.handle, c_string.as_ptr()) };
        if symbol.is_null() {
            return Err(LibraryLoaderError::symbol_not_found());
        }

        Ok(symbol)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { dlclose(self.handle) };
    }
}
