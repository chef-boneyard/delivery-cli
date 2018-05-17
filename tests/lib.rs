extern crate delivery;
extern crate log;
extern crate mockito;
extern crate serde_json;
extern crate tempdir;

// Thanks, Cargo.
macro_rules! test {
    ($name:ident $expr:expr) => {
        #[test]
        fn $name() {
            setup!();
            $expr;
        }
    };
}

macro_rules! panic_on_error {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => panic!("{:?}", e),
        }
    };
}

mod cli;
mod config;
mod delivery_config;
mod support;
mod user;
mod utils;
