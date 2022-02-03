use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

pub trait FromWideNull {
    fn from_wide_null(data: &[u16]) -> Self;
}

impl FromWideNull for OsString {
    fn from_wide_null(data: &[u16]) -> Self {
        let len = data.iter().take_while(|x| **x != 0).count();
        Self::from_wide(&data[..len])
    }
}
