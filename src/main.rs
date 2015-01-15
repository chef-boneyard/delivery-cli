#![allow(unstable)]
#![feature(plugin)]
extern crate regex;
#[plugin] #[no_link] extern crate regex_macros;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] extern crate docopt_macros;
#[macro_use] extern crate log;
extern crate term;
extern crate delivery;

use std::os;
use std::error;
use delivery::utils::say::{say, sayln};
use delivery::errors::{DeliveryError, Kind};
use delivery::config::Config;
use delivery::git;

docopt!(Args derive Show, "
Usage: delivery review [--for=<pipeline>]
       delivery checkout <change> [--for=<pipeline>] [--patchset=<number>]
       delivery diff <change> [--for=<pipeline>] [--patchset=<number>] [--local]
       delivery init [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--project=<project>]
       delivery setup [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--config-path=<dir>]
       delivery --help

Options:
  -h, --help               Show this message.
  -f, --for=<pipeline>     A pipeline to target
  -P, --patchset=<number>  A patchset number [default: latest]
  -u, --user=<user>        A delivery username
  -s, --server=<server>    A delivery server
  -e, --ent=<ent>          A delivery enterprise
  -o, --org=<org>          A delivery organization
  -p, --project=<project>  The project name
  -c, --config-path=<dir>  The directory to write a config to
  -l, --local              Diff against the local branch HEAD
  <change>                 The change to checkout
");

macro_rules! validate {
    ($config:ident, $value:ident) => (
        try!($config.clone().$value());
    )
}

#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    // debug!("{}", args);
    let cmd_result = match args {
        Args {
            cmd_review: true,
            flag_for: ref for_pipeline,
            ..
        } => review(&for_pipeline[]),
        Args {
            cmd_setup: true,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_config_path: ref path,
            ..
        } => setup(user.as_slice(), server.as_slice(), ent.as_slice(), org.as_slice(), path.as_slice()),
        Args {
            cmd_init: true,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_project: ref proj,
            ..
        } => init(user.as_slice(), server.as_slice(), ent.as_slice(), org.as_slice(), proj.as_slice()),
        Args {
            cmd_checkout: true,
            arg_change: ref change,
            flag_patchset: ref patchset,
            flag_for: ref pipeline,
            ..
        } => checkout(change.as_slice(), patchset.as_slice(), pipeline.as_slice()),
        Args {
            cmd_diff: true,
            arg_change: ref change,
            flag_patchset: ref patchset,
            flag_for: ref pipeline,
            flag_local: ref local,
            ..
        } => diff(change.as_slice(), patchset.as_slice(), pipeline.as_slice(), local),
        _ => no_matching_command(),
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}

fn cwd() -> Path {
    os::getcwd().unwrap()
}

fn no_matching_command() -> Result<(), DeliveryError> {
    Err(DeliveryError { kind: Kind::NoMatchingCommand, detail: None })
}

fn exit_with<T: error::Error>(e: T, i: isize) {
    sayln("red", e.description());
    match e.detail() {
        Some(deets) => sayln("red", deets.as_slice()),
        None => {}
    }
    os::set_exit_status(i)
}

fn setup(user: &str, server: &str, ent: &str, org: &str, path: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let config_path = if path.is_empty() {
        cwd()
    } else {
        Path::new(path)
    };
    let mut config = try!(Config::load_config(&config_path));
    config = config.set_server(server)
        .set_user(user)
        .set_enterprise(ent)
        .set_organization(org);
    try!(config.write_file(&config_path));
    Ok(())
}

fn init(user: &str, server: &str, ent: &str, org: &str, proj: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(Config::load_config(&cwd()));
    // Since we wind up taking the filename as a reference, we need to
    // have its scope be the entire method. Sadly, it means we call it
    // whether we need to or not. We could probably abstract this into
    // a function and get the lifetimes right, but.. meh :)
    let cwd = try!(os::getcwd());
    let final_proj = if proj.is_empty() {
        let cwd_name = cwd.filename().unwrap();
        std::str::from_utf8(cwd_name.clone()).unwrap()
    } else {
        proj
    };
    config = config.set_user(user)
        .set_server(server)
        .set_enterprise(ent)
        .set_organization(org)
        .set_project(final_proj);
    let u = validate!(config, user);
    let s = validate!(config, server);
    let e = validate!(config, enterprise);
    let o = validate!(config, organization);
    let p = validate!(config, project);
    try!(git::config_repo(
            u.as_slice(),
            s.as_slice(),
            e.as_slice(),
            o.as_slice(),
            p.as_slice()));
    sayln("white", "Configuration added!");
    Ok(())
}

fn review(for_pipeline: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_pipeline(for_pipeline);
    let target = validate!(config, pipeline);
    say("white", "Review for change  ");
    let head = try!(git::get_head());
    if for_pipeline == head.as_slice() {
        return Err(DeliveryError{ kind: Kind::CannotReviewSameBranch, detail: None })
    }
    say("yellow", head.as_slice());
    say("white", " targeted for pipeline ");
    sayln("magenta", target.as_slice());
    try!(git::git_push(head.as_slice(), target.as_slice()));
    Ok(())
}

fn checkout(change: &str, patchset: &str, pipeline: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_pipeline(pipeline);
    let target = validate!(config, pipeline);
    say("white", "Checking out ");
    say("yellow", change);
    say("white", " targeted for pipeline ");
    say("magenta", target.as_slice());

    if patchset == "latest" {
        sayln("white", " tracking latest changes");
    } else {
        say("white", " at patchset ");
        sayln("yellow", patchset);
    }
    try!(git::checkout_review(change, patchset, target.as_slice()));
    Ok(())
}

fn diff(change: &str, patchset: &str, pipeline: &str, local: &bool) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_pipeline(pipeline);
    let target = validate!(config, pipeline);
    say("white", "Showing diff for ");
    say("yellow", change);
    say("white", " targeted for pipeline ");
    say("magenta", target.as_slice());

    if patchset == "latest" {
        sayln("white", " latest patchset");
    } else {
        say("white", " at patchset ");
        sayln("yellow", patchset);
    }
    try!(git::diff(change, patchset, target.as_slice(), local));
    Ok(())
}

