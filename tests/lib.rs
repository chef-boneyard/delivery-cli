#![feature(path_ext)]
extern crate delivery;
extern crate rustc_serialize;
extern crate tempdir;
#[macro_use] extern crate log;

// Thanks, Cargo.
macro_rules! test {
    ($name:ident $expr:expr) => (
        #[test]
        fn $name() {
            setup();
            $expr;
        }
    )
}

macro_rules! panic_on_error {
    ($expr:expr) => (
        match $expr {
            Ok(val) => val,
            Err(e) => {
                panic!("{:?}", e)
            }
        }
    )
}

mod support;
mod config;
mod cli;
mod utils;
