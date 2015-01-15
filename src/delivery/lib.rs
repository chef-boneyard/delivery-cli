#![feature(plugin)]
extern crate regex;
#[plugin] #[no_link] extern crate regex_macros;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] extern crate docopt_macros;
#[macro_use] extern crate log;
extern crate term;
extern crate toml;

pub mod errors;
pub mod git;
pub mod utils;
pub mod config;
