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
#[macro_use] extern crate log;
extern crate term;
extern crate toml;
extern crate time;
extern crate rustc_serialize;
extern crate libc;
extern crate tempdir;
extern crate uuid;
#[macro_use] extern crate hyper;
extern crate mime;
extern crate clap;
extern crate crypto;

#[macro_export]
macro_rules! validate {
    ($config:ident, $value:ident) => (
        try!($config.$value());
    )
}

pub mod errors;
pub mod types;
pub mod git;
pub mod utils;
pub mod config;
pub mod delivery_config;
pub mod job;
pub mod getpass;
pub mod token;
pub mod http;
pub mod project;
pub mod cookbook;
pub mod cli;
pub mod command;
