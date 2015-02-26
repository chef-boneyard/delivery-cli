#![feature(io, path, core, os)]
extern crate delivery;
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
mod job;
mod cli;
