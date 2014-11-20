#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin, link)] extern crate log;

use std::os;
use std::error;
use utils::say::{say, sayln};
use errors::{DeliveryError, Kind};

pub mod errors;
pub mod git;
pub mod utils;

docopt!(Args deriving Show, "
Usage: delivery review [--for=<pipeline>]
       delivery checkout <change> [--patchset=<number>]
       delivery setup --user=<user> --server=<server> --ent=<ent> --org=<org> --project=<project>
       delivery --help

Options:
  -h, --help               Show this message.
  -f, --for=<pipeline>     A pipeline to target [default: master]
  -p, --patchset=<number>  A patchset number [default: latest]
  -u, --user=<user>        A delivery username
  -s, --server=<server>    A delivery server
  -e, --ent=<ent>          A delivery enterprise
  -o, --org=<org>          A delivery organization
  -p, --project=<project>  The project name
")

#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    debug!("{}", args);
    let cmd_result = match args {
        Args {
            cmd_review: true,
            flag_for: ref for_pipeline,
            ..
        } => review(for_pipeline.as_slice()),
        Args {
            cmd_setup: true,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_project: ref proj,
            ..
        } => setup(user.as_slice(), server.as_slice(), ent.as_slice(), org.as_slice(), proj.as_slice()),
        _ => no_matching_command(),
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}

fn no_matching_command() -> Result<(), DeliveryError> {
    Err(DeliveryError { kind: Kind::NoMatchingCommand, detail: None })
}

fn exit_with<T: error::Error>(e: T, i: int) {
    sayln("red", e.description());
    match e.detail() {
        Some(deets) => sayln("red", deets.as_slice()),
        None => {}
    }
    os::set_exit_status(i)
}

fn setup(user: &str, server: &str, ent: &str, org: &str, proj: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    try!(git::set_config(user, server, ent, org, proj));
    sayln("white", "Configuration added!");
    Ok(())
}

fn review(for_pipeline: &str) -> Result<(), DeliveryError> {
    let head = try!(git::get_head());
    if for_pipeline == head.as_slice() {
        return Err(DeliveryError{ kind: Kind::CannotReviewSameBranch, detail: None })
    }
    sayln("green", "Chef Delivery");
    say("white", "Review for change ");
    say("yellow", head.as_slice());
    say("white", " targeted for pipeline ");
    sayln("magenta", for_pipeline.as_slice());
    try!(git::git_push(head.as_slice(), for_pipeline));
    Ok(())
}

