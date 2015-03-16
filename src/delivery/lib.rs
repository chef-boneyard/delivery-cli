#![feature(plugin, collections, std_misc, core, old_io, path_ext)]
#![plugin(regex_macros, docopt_macros)]
extern crate regex;
#[no_link] extern crate regex_macros;
extern crate docopt;
#[no_link] extern crate docopt_macros;
#[macro_use] extern crate log;
extern crate term;
extern crate toml;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;
extern crate libc;
extern crate tempdir;
extern crate uuid;
extern crate hyper;
extern crate mime;

pub mod errors;
pub mod git;
pub mod utils;
pub mod config;
pub mod job;
pub mod getpass;
pub mod token;
pub mod http;

