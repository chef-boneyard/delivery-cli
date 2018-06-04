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

// #![feature(plugin, path_ext, convert)]
extern crate regex;
#[macro_use]
extern crate log;
extern crate serde;
extern crate term;
extern crate time;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate libc;
extern crate rpassword;
#[macro_use]
extern crate serde_json;
extern crate tempdir;
extern crate uuid;
#[macro_use]
extern crate hyper;
extern crate clap;
extern crate crypto;
extern crate mime;
#[cfg(test)]
extern crate mockito;

#[macro_export]
macro_rules! validate {
    ($config:ident, $value:ident) => {
        try!($config.$value());
    };
}

// Adding a quick macro to assert enums, specifically to test
// the error::Kind enum since we can't add #[derive(PartialEq)]
// because hyper and other types doesn't implement it
#[cfg(test)]
#[macro_export]
macro_rules! assert_enum {
    ($enum1:expr, $enum2:pat) => {
        match $enum1 {
            $enum2 => true,
            _ => false,
        }
    };
}

pub mod cli;
pub mod command;
pub mod config;
pub mod cookbook;
pub mod delivery_config;
pub mod errors;
pub mod fips;
pub mod getpass;
pub mod git;
pub mod http;
pub mod job;
pub mod json;
pub mod project;
pub mod token;
pub mod types;
pub mod user;
pub mod utils;
