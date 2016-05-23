//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//! A wrapper around libc's getpass function.
//!
//! This unsafe function allows you to read a value from stdin without
//! the input being echoed on the terminal. Expect this to work on OS
//! X and Linux only.

use std::os::raw::c_char;

#[cfg(not(target_os = "windows"))]
use std::ffi::{CString, CStr};

#[cfg(not(target_os = "windows"))]
use std::str;

#[cfg(target_os = "windows")]
use std::char;

#[cfg(not(target_os = "windows"))]
extern {
    fn getpass(pass: *const c_char) -> *const c_char;
}

#[cfg(target_os = "windows")]
extern {
    fn getch() -> c_char;
}

#[cfg(not(target_os = "windows"))]
pub fn read(prompt: &str) -> String {
    let cprompt = CString::new(prompt.as_bytes()).unwrap();
    let cresult = unsafe { getpass(cprompt.as_ptr()) };
    let bytes = unsafe { CStr::from_ptr(cresult).to_bytes() };
    str::from_utf8(bytes).unwrap().to_string()
}

#[cfg(target_os = "windows")]
pub fn read(prompt: &str) -> String {
    println!("{0}", prompt);
    let mut pass = String::new();
    let mut ch = read_char();
    while ch != '\r' {
        pass.push(ch);
        ch = read_char();
    }
    pass
}

#[cfg(target_os = "windows")]
fn read_char() -> char {
    let cresult = unsafe { getch() };
    char::from_u32(cresult as u32).unwrap()
}
