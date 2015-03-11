//! A wrapper around libc's getpass function.
//!
//! This unsafe function allows you to read a value from stdin without
//! the input being echoed on the terminal. Expect this to work on OS
//! X and Linux only.
use libc::types::os::arch::c95::c_char;
use std::ffi::{CString, CStr};
use std::str;

extern {
    fn getpass(pass: *const c_char) -> *const c_char;
}

pub fn read(prompt: &str) -> String {
    let cprompt = CString::new(prompt.as_bytes()).unwrap();
    let cresult = unsafe { getpass(cprompt.as_ptr()) };
    let bytes = unsafe { CStr::from_ptr(cresult).to_bytes() };
    str::from_utf8(bytes).unwrap().to_string()
}
