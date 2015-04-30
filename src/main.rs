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

#![feature(plugin, collections, exit_status)]
#![plugin(regex_macros, docopt_macros)]
extern crate regex;
#[no_link] extern crate regex_macros;
extern crate docopt;
#[no_link] extern crate docopt_macros;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate term;
extern crate delivery;
extern crate rustc_serialize;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use delivery::utils::{self, privileged_process};
// Allowing this, mostly just for testing.
#[allow(unused_imports)]
use delivery::utils::say::{self, say, sayln};
use delivery::errors::{DeliveryError, Kind};
use delivery::config::Config;
use delivery::delivery_config::DeliveryConfig;
use delivery::git;
use delivery::job::change::Change;
use delivery::job::workspace::{Workspace, Privilege};
use delivery::utils::path_join_many::PathJoinMany;
use delivery::getpass;
use delivery::token;
use delivery::http::{self, APIClient, APIAuth};
use delivery::project;

docopt!(Args derive Debug, "
Usage: delivery review [--for=<pipeline>] [--no-open]
       delivery clone <project> [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--git-url=<url>]
       delivery checkout <change> [--for=<pipeline>] [--patchset=<number>]
       delivery diff <change> [--for=<pipeline>] [--patchset=<number>] [--local]
       delivery init [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--project=<project>] [--type=<type>]
       delivery setup [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--config-path=<dir>] [--for=<pipeline>]
       delivery job <stage> <phase> [--change=<change>] [--for=<pipeline>] [--job-root=<dir>] [--project=<project>] [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--patchset=<number>] [--git-url=<url>] [--shasum=<gitsha>] [--change-id=<id>] [--no-spinner]
       delivery pipeline [--for=<pipeline>] [--user=<user>] [--server=<server>] [--ent=<ent>] [--org=<org>] [--project=<project>] [--config-path=<dir>]
       delivery api <method> <path> [--user=<user>] [--server=<server>] [--ent=<ent>] [--config-path=<dir>] [--data=<data>]
       delivery --help
       delivery token [--user=<user>] [--server=<server>] [--ent=<ent>]

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
  -C, --change=<change>    A delivery change branch name
  -i, --change-id=<id>     A delivery change ID
  -n, --no-spinner         Turn off the delightful spinner :(
  <change>                 A delivery change branch name
  <type>                   The type of project (currently supported: cookbook)
");

macro_rules! validate {
    ($config:ident, $value:ident) => (
        try!($config.clone().$value());
    )
}

#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    debug!("{:?}", args);
    let cmd_result = match args {
        Args {
            cmd_review: true,
            flag_for: ref for_pipeline,
            flag_no_open: ref no_open,
            ..
        } => review(&for_pipeline, &no_open),
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
            flag_type: ref proj_type,
            ..
        } => init(&user, &server, &ent, &org, &proj, &proj_type),
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
            cmd_pipeline: true,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_org: ref org,
            flag_project: ref proj,
            flag_for: ref pipeline,
            ..
        } => init_pipeline(&server, &user, &ent,
                           &org, &proj, &pipeline),
        Args {
            cmd_api: true,
            arg_method: ref method,
            arg_path: ref path,
            flag_user: ref user,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_data: ref data,
            ..
        } => api_req(&method, &path, &data,
                     &server, &ent,
                     &user),
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
            flag_patchset: ref patchset,
            flag_change_id: ref change_id,
            flag_git_url: ref git_url,
            flag_shasum: ref shasum,
            flag_no_spinner: no_spinner,
            ..
        } => {
            if no_spinner { say::turn_off_spinner() };
            job(&stage, &phase, &change, &pipeline, &job_root, &project, &user, &server, &ent, &org, &patchset, &change_id, &git_url, &shasum)
        },
        Args {
            cmd_token: true,
            flag_server: ref server,
            flag_ent: ref ent,
            flag_user: ref user,
            ..
        } => api_token(&server, &ent, &user),
        _ => no_matching_command(),
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}

#[allow(dead_code)]
fn cwd() -> PathBuf {
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
fn load_config(path: &PathBuf) -> Result<Config, DeliveryError> {
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
        PathBuf::from(path)
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
fn init(user: &str, server: &str, ent: &str, org: &str, proj: &str, proj_type: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    // Since we wind up taking the filename as a reference, we need to
    // have its scope be the entire method. Sadly, it means we call it
    // whether we need to or not. We could probably abstract this into
    // a function and get the lifetimes right, but.. meh :)
    let cwd = try!(env::current_dir());
    let filename = String::from_str(cwd.file_name().unwrap().to_str().unwrap());
    let final_proj = if proj.is_empty() {
        &filename[..]
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

    try!(project::import(&u, &s, &e, &o, &p, &cwd));

    // now to adding the .delivery/config.json
    try!(DeliveryConfig::init(&cwd, proj_type));
    // if we got here, we've checked out a feature branch, added a
    // config file, and made a local commit. Let's create the review!
    try!(review("master", &false));
    Ok(())
}

#[allow(dead_code)]
fn review(for_pipeline: &str, no_open: &bool) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(for_pipeline);
    let target = validate!(config, pipeline);
    say("white", "Review for change ");
    let head = try!(git::get_head());
    if &target == &head {
        return Err(DeliveryError{ kind: Kind::CannotReviewSameBranch, detail: None })
    }
    say("yellow", &head);
    say("white", " targeted for pipeline ");
    sayln("magenta", &target);
    let review = try!(git::git_push_review(&head, &target));
    for line in review.messages.iter() {
        sayln("white", line);
    }
    match review.url {
        Some(url) => {
            sayln("magenta", &url);
            if !no_open {
                try!(utils::open::item(&url));
            }
        },
        None => {}
    };
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
fn job(stage: &str,
       phase: &str,
       change: &str,
       pipeline: &str,
       job_root: &str,
       project: &str,
       user: &str,
       server: &str,
       ent: &str,
       org: &str,
       patchset: &str,
       change_id: &str,
       git_url: &str,
       shasum: &str) ->
Result<(), DeliveryError> { sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = if project.is_empty() {
        let filename = String::from_str(cwd().file_name().unwrap().to_str().unwrap());
        config.set_project(&filename)
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
        if privileged_process() {
            PathBuf::from("/var/opt/delivery/workspace").join_many(&[&s[..], &e, &o, &p, &pi, stage, phase])
        } else {
            match env::home_dir() {
                Some(path) => path.join_many(&[".delivery", &s, &e, &o, &p, &pi, stage, phase]),
                None => return Err(DeliveryError{ kind: Kind::NoHomedir, detail: None })
            }
        }
    } else {
        PathBuf::from(job_root)
    };
    let ws = Workspace::new(&job_root_path);
    sayln("white", "Creating workspace");
    try!(ws.build());
    say("white", "Cloning repository, and merging");
    let mut local = false;
    let patch = if patchset.is_empty() { "latest" } else { patchset };
    let c = if change.is_empty() {
        if shasum.is_empty() {
            local = true;
            let v = try!(git::get_head());
            say("yellow", &format!(" {}", &v));
            v
        } else {
            say("yellow", &format!(" {}", shasum));
            String::new()
        }
    } else {
        say("yellow", &format!(" {}", &change));
        format!("_reviews/{}/{}/{}", pi, change, patch)
    };
    say("white", " to ");
    sayln("magenta", &pi);
    let clone_url = if git_url.is_empty() {
        if local {
            cwd().into_os_string().to_string_lossy().into_owned()
        } else {
            git::delivery_ssh_url(&u, &s, &e, &o, &p)
        }
    } else {
        String::from_str(git_url)
    };
    try!(ws.setup_repo_for_change(&clone_url, &c, &pi, shasum));
    sayln("white", "Configuring the job");
    // This can be optimized out, almost certainly
    try!(utils::remove_recursive(&ws.chef.join("build_cookbook")));
    let change = Change{
        enterprise: e.to_string(),
        organization: o.to_string(),
        project: p.to_string(),
        pipeline: pi.to_string(),
        stage: stage.to_string(),
        phase: phase.to_string(),
        git_url: clone_url.to_string(),
        sha: shasum.to_string(),
        patchset_branch: c.to_string(),
        change_id: change_id.to_string(),
        patchset_number: patch.to_string()
    };
    try!(ws.setup_chef_for_job(&u, &s, change));
    sayln("white", "Running the job");
    if privileged_process() {
        sayln("yellow", "Setting up the builder");
        try!(ws.run_job("default", Privilege::NoDrop));
        sayln("magenta", &format!("Running phase {}", phase));
        try!(ws.run_job(phase, Privilege::Drop));
    } else {
        try!(ws.run_job(phase, Privilege::NoDrop));
    }
    Ok(())
}

#[allow(dead_code)]
fn api_token(server: &str, ent: &str,
             user: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_server(server)
        .set_enterprise(ent)
        .set_user(user);
    let s = validate!(config, server);
    let e = validate!(config, enterprise);
    let u = validate!(config, user);

    let mut tstore = try!(token::TokenStore::from_home());
    let pass = getpass::read("Delivery password: ");
    let token = try!(http::token::request(&s, &e, &u, &pass));
    sayln("magenta", &format!("token: {}", &token));
    try!(tstore.write_token(&s, &e, &u, &token));
    sayln("green", &format!("saved API token to: {}", tstore.path().display()));
    Ok(())
}

#[allow(dead_code)]
fn init_pipeline(server: &str, user: &str,
                 ent: &str, org: &str, proj: &str,
                 pipeline: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery: baking a new pipeline");
    let mut config = try!(Config::load_config(&cwd()));
    let cwd = try!(env::current_dir());
    let final_proj = if proj.is_empty() {
        // let cwd_name = cwd.file_name().unwrap().to_str().unwrap();
        // std::str::from_utf8(cwd_name.clone()).unwrap()
        cwd.file_name().unwrap().to_str().unwrap()
    } else {
        proj
    };
    config = config.set_user(user)
        .set_server(server)
        .set_enterprise(ent)
        .set_organization(org)
        .set_project(final_proj);
    let p = validate!(config, project);
    let _ = validate!(config, user);
    let _ = validate!(config, server);
    let e = validate!(config, enterprise);
    let o = validate!(config, organization);
    say("white", &format!("hello, pipeline {}\n", pipeline));
    sayln("white", &format!("e: {} o: {} p: {}", e, o, p));
    // create the project
    // setup the remote
    // push master
    // create the pipeline
    // checkout a feature branch
    // add a config file and commit it
    // push the review
    // maybe back to master?
    Ok(())
}

#[allow(dead_code)]
fn api_req(method: &str, path: &str, data: &str,
           server: &str, ent: &str, user: &str) -> Result<(), DeliveryError> {
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_user(user)
        .set_server(server)
        .set_enterprise(ent);
    let u = validate!(config, user);
    let s = validate!(config, server);
    let e = validate!(config, enterprise);

    let mut client = APIClient::new_https(&s, &e);

    let tstore = try!(token::TokenStore::from_home());
    
    let auth = try!(APIAuth::from_token_store(tstore, &s, &e, &u));
    client.set_auth(auth);
    let result = match method {
        "get" => client.get(path),
        "post" => client.post(path, data),
        _ => return Err(DeliveryError{ kind: Kind::UnsupportedHttpMethod,
                                       detail: None })
    };
    match APIClient::extract_pretty_json(result) {
        Ok(v) => println!("{}", v),
        Err(e) => return Err(DeliveryError{ kind: Kind::JsonParseError,
                                            detail: Some(e) })
    };
    Ok(())
}
