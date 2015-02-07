#![feature(plugin, collections, path, env, core, io)]
extern crate regex;
#[plugin] #[no_link] extern crate regex_macros;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;
#[macro_use] extern crate log;
extern crate term;
extern crate delivery;
extern crate "rustc-serialize" as rustc_serialize;

use std::env;
use std::error::Error;
use std::old_io::{self, fs};
use delivery::utils::say::{say, sayln};
use delivery::errors::{DeliveryError, Kind};
use delivery::config::Config;
use delivery::git;
use delivery::job::workspace::Workspace;

docopt!(Args derive Debug, "
Usage: delivery review [--for=<pipeline>]
       delivery clone <project> [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--git-url=<url>]
       delivery checkout <change> [--for=<pipeline>] [--patchset=<number>]
       delivery diff <change> [--for=<pipeline>] [--patchset=<number>] [--local]
       delivery init [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--project=<project>]
       delivery setup [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--config-path=<dir>] [--for=<pipeline>]
       delivery job <stage> <phase> [--change=<change>] [--for=<pipeline>] [--job-root=<dir>] [--project=<project>] [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--git-url=<url>] [--shasum=<gitsha>]
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
  -g, --git-url=<url>      A raw git URL
  -j, --job-root=<path>    The path to the job root
  -S, --shasum=<gitsha>    A Git SHA
  -c, --change=<change>    A delivery change branch name
  <change>                 A delivery change branch name
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
        } => review(&for_pipeline),
        Args {
            cmd_setup: true,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_config_path: ref path,
            flag_for: ref pipeline,
            ..
        } => setup(&user, &server, &ent, &org, &path, &pipeline),
        Args {
            cmd_init: true,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_project: ref proj,
            ..
        } => init(&user, &server, &ent, &org, &proj),
        Args {
            cmd_checkout: true,
            arg_change: ref change,
            flag_patchset: ref patchset,
            flag_for: ref pipeline,
            ..
        } => checkout(&change, &patchset, &pipeline),
        Args {
            cmd_diff: true,
            arg_change: ref change,
            flag_patchset: ref patchset,
            flag_for: ref pipeline,
            flag_local: ref local,
            ..
        } => diff(&change, &patchset, &pipeline, local),
        Args {
            cmd_clone: true,
            arg_project: ref project,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_git_url: ref git_url,
            ..
        } => clone(&project, &user, &server, &ent, &org, &git_url),
        Args {
            cmd_job: true,
            arg_stage: ref stage,
            arg_phase: ref phase,
            flag_change: ref change,
            flag_for: ref pipeline,
            flag_job_root: ref job_root,
            flag_project: ref project,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_git_url: ref git_url,
            flag_shasum: ref shasum,
            ..
        } => job(&stage, &phase, &change, &pipeline, &job_root, &project, &user, &server, &ent, &org, &git_url, &shasum),
        _ => no_matching_command(),
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}

#[allow(dead_code)]
fn cwd() -> Path {
    env::current_dir().unwrap()
}

#[allow(dead_code)]
fn no_matching_command() -> Result<(), DeliveryError> {
    Err(DeliveryError { kind: Kind::NoMatchingCommand, detail: None })
}

#[allow(dead_code)]
fn exit_with(e: DeliveryError, i: isize) {
    sayln("red", e.description());
    match e.detail() {
        Some(deets) => sayln("red", &deets),
        None => {}
    }
    let x = i as i32;
    env::set_exit_status(x)
}

#[allow(dead_code)]
fn load_config(path: &Path) -> Result<Config, DeliveryError> {
    say("white", "Loading configuration from ");
    let msg = format!("{}", path.display());
    sayln("yellow", &msg);
    let config = try!(Config::load_config(&cwd()));
    Ok(config)
}

#[allow(dead_code)]
fn setup(user: &str, server: &str, ent: &str, org: &str, path: &str, pipeline: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let config_path = if path.is_empty() {
        cwd()
    } else {
        Path::new(path)
    };
    let mut config = try!(load_config(&config_path));
    config = config.set_server(server)
        .set_user(user)
        .set_enterprise(ent)
        .set_organization(org)
        .set_pipeline(pipeline) ;
    try!(config.write_file(&config_path));
    Ok(())
}

#[allow(dead_code)]
fn init(user: &str, server: &str, ent: &str, org: &str, proj: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    // Since we wind up taking the filename as a reference, we need to
    // have its scope be the entire method. Sadly, it means we call it
    // whether we need to or not. We could probably abstract this into
    // a function and get the lifetimes right, but.. meh :)
    let cwd = try!(env::current_dir());
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
            &u,
            &s,
            &e,
            &o,
            &p,
            &cwd));
    sayln("white", "Configuration added!");
    Ok(())
}

#[allow(dead_code)]
fn review(for_pipeline: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(for_pipeline);
    let target = validate!(config, pipeline);
    say("white", "Review for change  ");
    let head = try!(git::get_head());
    if &target == &head {
        return Err(DeliveryError{ kind: Kind::CannotReviewSameBranch, detail: None })
    }
    say("yellow", &head);
    say("white", " targeted for pipeline ");
    sayln("magenta", &target);
    try!(git::git_push(&head, &target));
    Ok(())
}

#[allow(dead_code)]
fn checkout(change: &str, patchset: &str, pipeline: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(pipeline);
    let target = validate!(config, pipeline);
    say("white", "Checking out ");
    say("yellow", change);
    say("white", " targeted for pipeline ");
    say("magenta", &target);

    if patchset == "latest" {
        sayln("white", " tracking latest changes");
    } else {
        say("white", " at patchset ");
        sayln("yellow", patchset);
    }
    try!(git::checkout_review(change, patchset, &target));
    Ok(())
}

#[allow(dead_code)]
fn diff(change: &str, patchset: &str, pipeline: &str, local: &bool) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(pipeline);
    let target = validate!(config, pipeline);
    say("white", "Showing diff for ");
    say("yellow", change);
    say("white", " targeted for pipeline ");
    say("magenta", &target);

    if patchset == "latest" {
        sayln("white", " latest patchset");
    } else {
        say("white", " at patchset ");
        sayln("yellow", patchset);
    }
    try!(git::diff(change, patchset, &target, local));
    Ok(())
}

#[allow(dead_code)]
fn clone(project: &str, user: &str, server: &str, ent: &str, org: &str, git_url: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_user(user)
        .set_server(server)
        .set_enterprise(ent)
        .set_organization(org);
    let u = validate!(config, user);
    let s = validate!(config, server);
    let e = validate!(config, enterprise);
    let o = validate!(config, organization);
    say("white", "Cloning ");
    let clone_url = if git_url.is_empty() {
        say("yellow", &format!("{}/{}/{}", e, o, project));
        git::delivery_ssh_url(&u, &s, &e, &o, project)
    } else {
        say("yellow", git_url);
        String::from_str(git_url)
    };
    say("white", " to ");
    sayln("magenta", &format!("{}", project));
    try!(git::clone(project, &clone_url));
    let project_root = cwd().join(project);
    try!(git::config_repo(&u,
                          &s,
                          &e,
                          &o,
                          project,
                          &project_root));
    Ok(())
}

#[allow(dead_code)]
fn job(stage: &str, phase: &str, change: &str, pipeline: &str, job_root: &str, project: &str, user: &str, server: &str, ent: &str, org: &str, git_url: &str, shasum: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = if project.is_empty() {
        config.set_project(&String::from_utf8_lossy(cwd().filename().unwrap()))
    } else {
        config.set_project(project)
    };
    config = config.set_pipeline(pipeline)
        .set_user(user)
        .set_server(server)
        .set_enterprise(ent)
        .set_organization(org);
    let p = validate!(config, project);
    let u = validate!(config, user);
    let s = validate!(config, server);
    let e = validate!(config, enterprise);
    let o = validate!(config, organization);
    let pi = validate!(config, pipeline);
    say("white", "Starting job for ");
    say("green", &format!("{}", &p));
    say("yellow", &format!(" {}", stage));
    sayln("magenta", &format!(" {}", phase));
    let job_root_path = if job_root.is_empty() {
        let homedir_path = match env::home_dir() {
            Some(path) => path.join_many(&[".delivery", &s, &e, &o, &p, &pi, stage, phase]),
            None => return Err(DeliveryError{ kind: Kind::NoHomedir, detail: None })
        };
        try!(fs::mkdir_recursive(&homedir_path, old_io::USER_RWX));
        homedir_path
    } else {
        Path::new(job_root)
    };
    let ws = Workspace::new(&job_root_path);
    sayln("white", "Creating workspace");
    try!(ws.build());
    say("white", "Cloning repository, and merging ");
    let mut local = false;
    let c = if change.is_empty() {
        if shasum.is_empty() {
            local = true;
            let v = try!(git::get_head());
            say("yellow", &v);
            v
        } else {
            say("yellow", shasum);
            String::new()
        }
    } else {
        say("yellow", &change);
        format!("_reviews/{}/{}/latest", pi, change)
    };
    say("white", " to ");
    sayln("magenta", &pi);
    let clone_url = if git_url.is_empty() {
        if local {
            String::from_str(cwd().as_str().unwrap())
        } else {
            git::delivery_ssh_url(&u, &s, &e, &o, &p)
        }
    } else {
        String::from_str(git_url)
    };
    try!(ws.setup_repo_for_change(&clone_url, &c, &pi, shasum));
    sayln("white", "Configuring the job");
    try!(ws.setup_chef_for_job());
    sayln("white", "Running the job");
    try!(ws.run_job(phase));
    Ok(())
}
