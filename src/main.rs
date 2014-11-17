#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate git2;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin, link)] extern crate log;

use std::os;
use std::error;
use utils::say::{say, sayln};
use errors::{DeliveryError};

pub mod errors;
pub mod git;
pub mod utils;

docopt!(Args deriving Show, "
Usage: delivery review [--for=<pipeline>]
       delivery checkout <change> [--patchset=<number>]
       delivery rebase [--for=<pipeline>]
       delivery --help

Options:
  -h, --help               Show this message.
  -f, --for=<pipeline>     A pipeline to target [default: master]
  -p, --patchset=<number>  A patchset number [default: latest]
")

#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    debug!("{}", args);
    let cmd_result = match args {
        Args {
            cmd_review: true, flag_for: ref for_pipeline, ..
        } => review(for_pipeline.as_slice()),
        _ => no_matching_command(),
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}


fn no_matching_command() -> Result<bool, DeliveryError> {
    Err(DeliveryError { kind: errors::NoMatchingCommand, detail: None })
}

fn exit_with<T: error::Error>(e: T, i: int) {
    sayln("red", e.description());
    match e.detail() {
        Some(deets) => sayln("red", deets.as_slice()),
        None => {}
    }
    os::set_exit_status(i)
}

fn review(for_pipeline: &str) -> Result<bool, DeliveryError> {
    let repo = try!(git::get_repository());
    let head = try!(git::get_head(repo));
    if for_pipeline == head.as_slice() {
        return Err(DeliveryError{ kind: errors::CannotReviewSameBranch, detail: None })
    }
    say("green", "Delivery");
    say("white", " review for change ");
    say("yellow", head.as_slice());
    say("white", " targeted for pipeline ");
    sayln("magenta", for_pipeline.as_slice());
    try!(git::git_push(head.as_slice(), for_pipeline));
    Ok(true)
}

