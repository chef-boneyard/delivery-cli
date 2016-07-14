//
// Copyright:: Copyright (c) 2016 Chef Software, Inc.
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

use std::env;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::error::Error;
use std::path::PathBuf;
use std::io::prelude::*;
use std::time::Duration;
use std;
use token::TokenStore;
use utils::{self, cwd, privileged_process};
use utils::say::{self, sayln, say};
use errors::{DeliveryError, Kind};
use types::{ExitCode};
use config::Config;
use git;
use job::change::Change;
use job::workspace::{Workspace, Privilege};
use utils::path_join_many::PathJoinMany;
use http::APIClient;
use hyper::status::StatusCode;
use clap::{Arg, App, ArgMatches};

#[macro_use(validate)]

macro_rules! make_arg_vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(Arg::from_usage($x));
            )*
            temp_vec
        }
    };
}

macro_rules! fn_arg {
    ( $fn_name:ident, $usage:expr ) => (
        fn $fn_name<'a>() -> Arg<'a, 'a> {
            Arg::from_usage($usage)
        }
    )
}

// Modules for setting up clap subcommand including their options and defaults,
// as well as advanced subcommand match parsing (see local for an example).
mod api;
pub mod review;
mod checkout;
mod clone;
mod diff;
pub mod init;
mod job;
mod spin;
mod token;
mod setup;
pub mod local;

// Implemented sub-commands. Should handle everything after args have
// been parsed, including running the command, error handling, and UI outputting.
use command;

fn u_e_s_o_args<'a>() -> Vec<Arg<'a, 'a>> {
    make_arg_vec![
        "-u --user=[user] 'User name for Delivery authentication'",
        "-e --ent=[ent] 'The enterprise in which the project lives'",
        "-o --org=[org] 'The organization in which the project lives'",
        "-s --server=[server] 'The Delivery server address'"]
}

fn scp_args<'a>() -> Vec<Arg<'a, 'a>> {
    make_arg_vec![
        "--bitbucket=[project-key] 'Use a Bitbucket repository for Code Review with the provided Project Key'",
        "--github=[org-name] 'Use a Github repository for Code Review with the provided Organization'",
        "-r --repo-name=[repo-name] 'Source code provider repository name'",
        "--no-verify-ssl 'Do not use SSL verification. [Github]'"]
}

fn_arg!(config_project_arg,
       "-c --config-json=[config-json] 'Path of a custom config.json file'");

fn_arg!(for_arg,
       "-f --for=[pipeline] 'Target pipeline for change (default: master)'");

fn_arg!(patchset_arg,
       "-P --patchset=[patchset] 'A patchset number (default: latest)'");

fn_arg!(project_arg,
       "-p --project=[project] 'The project name'");

fn_arg!(config_path_arg,
        "--config-path=[dir] 'Directory to read/write your config file \
         (cli.toml) from'");

fn_arg!(local_arg, "-l --local 'Operate without a Delivery server'");

fn_arg!(no_open_arg, "-n --no-open 'Do not open the change in a browser'");

fn_arg!(auto_bump, "-a --auto-bump 'Automatic cookbook version bump'");

fn_arg!(no_spinner_arg, "--no-spinner 'Disable the spinner'");

fn_arg!(non_interactive_arg, "--non-interactive 'Disable cli interactions'");

pub fn run() {
    let build_version = format!("{} {}", version(), build_git_sha());

    let app = make_app(&build_version);
    let app_matches = app.get_matches();

    let cmd_result = match app_matches.subcommand() {
        (api::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            api_req(&api::ApiClapOptions::new(matches))
        },
        (checkout::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            checkout(&checkout::CheckoutClapOptions::new(matches))
        },
        (clone::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            clone(&clone::CloneClapOptions::new(matches))
        },
        (diff::SUBCOMMAND_NAME, Some(matches)) => {
            diff(&diff::DiffClapOptions::new(&matches))
        },
        (init::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            let init_opts = init::InitClapOptions::new(&matches);
            command::init::run(init_opts)
        },
        (job::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            job(&job::JobClapOptions::new(&matches))
        },
        (review::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            let review_opts = review::ReviewClapOptions::new(matches);
            command::review::run(review_opts)
        },
        (setup::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            setup(&setup::SetupClapOptions::new(&matches))
        },
        (token::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            token(&token::TokenClapOptions::new(&matches))
        },
        (local::SUBCOMMAND_NAME, Some(matches)) => {
            let local_opts = local::LocalClapOptions::new(matches);
            command::local::run(local_opts)
        },
        (spin::SUBCOMMAND_NAME, Some(matches)) => {
            handle_spinner(&matches);
            let spin_opts = spin::SpinClapOptions::new(&matches);
            let spinner = utils::say::Spinner::start();
            let sleep_time = Duration::from_secs(spin_opts.time);
            std::thread::sleep(sleep_time);
            spinner.stop();
            handle_spinner(&matches);
            Ok(0)
        },
        _ => {
            // ownership issue with use of above defined app
            // so for now...
            let mut a = make_app(&build_version);
            a.print_help().ok().expect("failed to write help to stdout");
            sayln("red", "missing subcommand");
            process::exit(1);
        }
    };
    match cmd_result {
        // You can exit with any integer, can also be used to bypass default
        // error handling if you handled an error and returned non-zero.
        Ok(exit_status) => process::exit(exit_status),
        // Handles DeliveryError and exits 1.
        Err(e) => exit_with(e, 1)
    }
}

fn make_app<'a>(version: &'a str) -> App<'a, 'a> {
    App::new("delivery")
        .version(version)
        .arg(no_spinner_arg().global(true))
        .arg(non_interactive_arg().global(true))
        .subcommand(review::clap_subcommand())
        .subcommand(clone::clap_subcommand())
        .subcommand(checkout::clap_subcommand())
        .subcommand(diff::clap_subcommand())
        .subcommand(init::clap_subcommand())
        .subcommand(setup::clap_subcommand())
        .subcommand(job::clap_subcommand())
        .subcommand(api::clap_subcommand())
        .subcommand(token::clap_subcommand())
        .subcommand(spin::clap_subcommand())
        .subcommand(local::clap_subcommand())
}

fn handle_spinner(matches: &ArgMatches) {
    if matches.is_present("no-spinner") {
        say::turn_off_spinner()
    };
}

fn exit_with(e: DeliveryError, i: isize) {
    sayln("red", e.description());
    match e.detail() {
        Some(deets) => sayln("red", &deets),
        None => {}
    }
    let x = i as ExitCode;
    process::exit(x)
}

pub fn load_config(path: &PathBuf) -> Result<Config, DeliveryError> {
    say("white", "Loading configuration from ");
    let msg = format!("{}", path.display());
    sayln("yellow", &msg);
    let config = try!(Config::load_config(&cwd()));
    Ok(config)
}

fn setup(opts: &setup::SetupClapOptions) -> Result<ExitCode, DeliveryError> {
    sayln("green", "Chef Delivery");
    let config_path = if opts.path.is_empty() {
        cwd()
    } else {
        PathBuf::from(opts.path)
    };
    let mut config = try!(load_config(&config_path));
    config = config.set_server(opts.server)
        .set_user(opts.user)
        .set_enterprise(opts.ent)
        .set_organization(opts.org)
        .set_pipeline(opts.pipeline) ;
    try!(config.write_file(&config_path));
    Ok(0)
}

fn checkout(opts: &checkout::CheckoutClapOptions) -> Result<ExitCode, DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(opts.pipeline);
    let target = validate!(config, pipeline);
    say("white", "Checking out ");
    say("yellow", opts.change);
    say("white", " targeted for pipeline ");
    say("magenta", &target);

    let pset = match opts.patchset {
        "" | "latest" => {
            sayln("white", " tracking latest changes");
            "latest"
        },
        p @ _ => {
            say("white", " at patchset ");
            sayln("yellow", p);
            p
        }
    };
    try!(git::checkout_review(opts.change, pset, &target));
    Ok(0)
}

fn diff(opts: &diff::DiffClapOptions) ->  Result<ExitCode, DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(opts.pipeline);
    let target = validate!(config, pipeline);
    say("white", "Showing diff for ");
    say("yellow", opts.change);
    say("white", " targeted for pipeline ");
    say("magenta", &target);

    if opts.patchset == "latest" {
        sayln("white", " latest patchset");
    } else {
        say("white", " at patchset ");
        sayln("yellow", opts.patchset);
    }
    try!(git::diff(opts.change, opts.patchset, &target, &opts.local));
    Ok(0)
}

fn clone(opts: &clone::CloneClapOptions) -> Result<ExitCode, DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_user(opts.user)
        .set_server(opts.server)
        .set_enterprise(opts.ent)
        .set_organization(opts.org)
        .set_project(opts.project);
    say("white", "Cloning ");
    let delivery_url = try!(config.delivery_git_ssh_url());
    let clone_url = if opts.git_url.is_empty() {
        delivery_url.clone()
    } else {
        String::from(opts.git_url)
    };
    say("yellow", &clone_url);
    say("white", " to ");
    sayln("magenta", &format!("{}", opts.project));
    try!(git::clone(opts.project, &clone_url));
    let project_root = cwd().join(opts.project);
    try!(git::config_repo(&delivery_url,
                          &project_root));
    Ok(0)
}

fn job(opts: &job::JobClapOptions) -> Result<ExitCode, DeliveryError> {
    sayln("green", "Chef Delivery");
    if !opts.docker_image.is_empty() {
        // The --docker flag was specified, let's do this!
        let cwd_path = cwd();
        let cwd_str = cwd_path.to_str().unwrap();
        let volume = &[cwd_str, cwd_str].join(":");
        // We might want to wrap this in `bash -c $BLAH 2>&1` so that
        // we get stderr with our streaming output. OTOH, what's here
        // seems to work in terms of expected output and has a better
        // chance of working on Windows.
        let mut docker = utils::make_command("docker");

        docker.arg("run")
            .arg("-t")
            .arg("-i")
            .arg("-v").arg(volume)
            .arg("-w").arg(cwd_str)
            // TODO: get this via config
            .arg("--dns").arg("8.8.8.8")
            .arg(opts.docker_image)
            .arg("delivery").arg("job").arg(opts.stage).arg(opts.phases);

        let flags_with_values = vec![("--change", opts.change),
                                     ("--for", opts.pipeline),
                                     ("--job-root", opts.job_root),
                                     ("--project", opts.project),
                                     ("--user", opts.user),
                                     ("--server", opts.server),
                                     ("--ent", opts.ent),
                                     ("--org", opts.org),
                                     ("--patchset", opts.patchset),
                                     ("--change_id", opts.change_id),
                                     ("--git-url", opts.git_url),
                                     ("--shasum", opts.shasum),
                                     ("--branch", opts.branch)];

        for (flag, value) in flags_with_values {
            maybe_add_flag_value(&mut docker, flag, value);
        }

        let flags = vec![("--skip-default", &opts.skip_default),
                         ("--local", &opts.local)];

        for (flag, value) in flags {
            maybe_add_flag(&mut docker, flag, value);
        }

        docker.stdout(Stdio::piped());
        docker.stderr(Stdio::piped());

        debug!("command: {:?}", docker);
        let mut child = try!(docker.spawn());
        let mut c_stdout = match child.stdout {
            Some(ref mut s) => s,
            None => {
                let msg = "failed to execute docker".to_string();
                let docker_err = DeliveryError { kind: Kind::FailedToExecute,
                                                 detail: Some(msg) };
                return Err(docker_err);
            }
        };
        let mut line = String::with_capacity(256);
        loop {
            let mut buf = [0u8; 1]; // Our byte buffer
            let len = try!(c_stdout.read(&mut buf));
            match len {
                0 => { // 0 == EOF, so stop writing and finish progress
                    break;
                },
                _ => { // Write the buffer to the BufWriter on the Heap
                    let buf_vec = buf[0 .. len].to_vec();
                    let buf_string = String::from_utf8(buf_vec).unwrap();
                    line.push_str(&buf_string);
                    if line.contains("\n") {
                        print!("{}", line);
                        line = String::with_capacity(256);
                    }
                }
            }
        }
        return Ok(0);
    }

    let mut config = try!(load_config(&cwd()));
    config = if opts.project.is_empty() {
        let filename = String::from(cwd().file_name().unwrap().to_str().unwrap());
        config.set_project(&filename)
    } else {
        config.set_project(opts.project)
    };

    config = config.set_pipeline(opts.pipeline)
        .set_user(with_default(opts.user, "you", &opts.local))
        .set_server(with_default(opts.server, "localhost", &opts.local))
        .set_enterprise(with_default(opts.ent, "local", &opts.local))
        .set_organization(with_default(opts.org, "workstation", &opts.local));
    let p = try!(config.project());
    let s = try!(config.server());
    let e = try!(config.enterprise());
    let o = try!(config.organization());
    let pi = try!(config.pipeline());
    say("white", "Starting job for ");
    say("green", &format!("{}", &p));
    say("yellow", &format!(" {}", opts.stage));
    sayln("magenta", &format!(" {}", opts.phases));
    let phases: Vec<&str> = opts.phases.split(" ").collect();
    let phase_dir = phases.join("-");
    // Builder nodes are expected to be running this command via
    // push-jobs-client as root and set $HOME to the workspace location.
    // If this process is not running as root via push-jobs-client, we'll
    // append ".delivery" to the user's $HOME location and use that as the
    // workspace path to avoid writing our working files directly into $HOME.
    let ws_path = match env::home_dir() {
        Some(path) => if privileged_process() {
                          PathBuf::from(path)
                      } else {
                          PathBuf::from(path).join_many(&[".delivery"])
                      },
        None => return Err(DeliveryError{ kind: Kind::NoHomedir, detail: None })
    };
    debug!("Workspace Path: {}", ws_path.display());
    let job_root_path = if opts.job_root.is_empty() {
        let phase_path: &[&str] = &[&s[..], &e, &o, &p, &pi, opts.stage, &phase_dir];
        ws_path.join_many(phase_path)
    } else {
        PathBuf::from(opts.job_root)
    };
    let ws = Workspace::new(&job_root_path);
    sayln("white", &format!("Creating workspace in {}", job_root_path.to_string_lossy()));
    try!(ws.build());
    say("white", "Cloning repository, and merging");
    let mut local_change = false;
    let patch = if opts.patchset.is_empty() { "latest" } else { opts.patchset };
    let c = if ! opts.branch.is_empty() {
        say("yellow", &format!(" {}", &opts.branch));
        String::from(opts.branch)
    } else if ! opts.change.is_empty() {
        say("yellow", &format!(" {}", &opts.change));
        format!("_reviews/{}/{}/{}", pi, opts.change, patch)
    } else if ! opts.shasum.is_empty() {
        say("yellow", &format!(" {}", opts.shasum));
        String::new()
    } else {
        local_change = true;
        let v = try!(git::get_head());
        say("yellow", &format!(" {}", &v));
        v
    };
    say("white", " to ");
    sayln("magenta", &pi);
    let clone_url = if opts.git_url.is_empty() {
        if local_change {
            cwd().into_os_string().to_string_lossy().into_owned()
        } else {
            try!(config.delivery_git_ssh_url())
        }
    } else {
        String::from(opts.git_url)
    };
    try!(ws.setup_repo_for_change(&clone_url, &c, &pi, opts.shasum));
    sayln("white", "Configuring the job");
    // This can be optimized out, almost certainly
    try!(utils::remove_recursive(&ws.chef.join("build_cookbook")));
    let change = Change{
        enterprise: e.to_string(),
        organization: o.to_string(),
        project: p.to_string(),
        pipeline: pi.to_string(),
        stage: opts.stage.to_string(),
        phase: opts.phases.to_string(),
        git_url: clone_url.to_string(),
        sha: opts.shasum.to_string(),
        patchset_branch: c.to_string(),
        change_id: opts.change_id.to_string(),
        patchset_number: patch.to_string()
    };
    try!(ws.setup_chef_for_job(&config, change, &ws_path));
    sayln("white", "Running the job");

    let privilege_drop = if privileged_process() {
        Privilege::Drop
    } else {
        Privilege::NoDrop
    };

    if privileged_process() && !&opts.skip_default {
        sayln("yellow", "Setting up the builder");
        try!(ws.run_job("default", &Privilege::NoDrop, &local_change));
    }

    let phase_msg = if phases.len() > 1 {
        "phases"
    } else {
        "phase"
    };
    sayln("magenta", &format!("Running {} {}", phase_msg, phases.join(", ")));
    try!(ws.run_job(opts.phases, &privilege_drop, &local_change));
    Ok(0)
}

fn maybe_add_flag_value(cmd: &mut Command, flag: &str, value: &str) {
    if !value.is_empty() {
        cmd.arg(flag).arg(value);
    }
}

fn maybe_add_flag(cmd: &mut Command, flag: &str, value: &bool) {
    if *value {
        cmd.arg(flag);
    }
}

fn with_default<'a>(val: &'a str, default: &'a str, local: &bool) -> &'a str {
    if !local || !val.is_empty() {
        val
    } else {
        default
    }
}

fn token(opts: &token::TokenClapOptions) -> Result<ExitCode, DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_server(opts.server)
        .set_api_port(opts.port)
        .set_enterprise(opts.ent)
        .set_user(opts.user);
    if opts.saml.is_some() {
        config.saml = opts.saml;
    }
    if opts.verify {
        try!(TokenStore::verify_token(&config));
    } else {
        try!(TokenStore::request_token(&config));
    }
    Ok(0)
}

fn version() -> String {
    let build_version = option_env!("DELIV_CLI_VERSION").unwrap_or("0.0.0");
    format!("{}", build_version)
}

fn build_git_sha() -> String {
    let sha = option_env!("DELIV_CLI_GIT_SHA").unwrap_or("0000");
    format!("({})", sha)
}

fn api_req(opts: &api::ApiClapOptions) -> Result<ExitCode, DeliveryError> {
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_user(opts.user)
        .set_server(opts.server)
        .set_api_port(opts.api_port)
        .set_enterprise(opts.ent);
    let client = try!(APIClient::from_config(&config));
    let mut result = match opts.method {
        "get" => try!(client.get(opts.path)),
        "post" => try!(client.post(opts.path, opts.data)),
        "put" => try!(client.put(opts.path, opts.data)),
        "delete" => try!(client.delete(opts.path)),
        _ => return Err(DeliveryError{ kind: Kind::UnsupportedHttpMethod,
                                       detail: None })
    };
    match result.status {
        StatusCode::NoContent => {},
        StatusCode::InternalServerError => {
            return Err(DeliveryError{ kind: Kind::InternalServerError, detail: None})
        },
        _ => {
            let pretty_json = try!(APIClient::extract_pretty_json(&mut result));
            println!("{}", pretty_json);
        }
    };
    Ok(0)
}

fn value_of<'a>(matches: &'a ArgMatches, key: &str) -> &'a str {
    matches.value_of(key).unwrap_or("")
}


#[cfg(test)]
mod tests {
    use cli;
    use cli::{api, review, clone, checkout, diff, init, job, spin, token, setup};

    #[test]
    fn test_clap_api_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "api", "get", "endpoint",
                                           "--data", "\"name\":\"n\",\"value\":\"d\"",
                                           "-e", "starwars", "-u", "vader", "-s",
                                           "death-star", "--api-port", "9999"]);
        assert_eq!(Some("api"), matches.subcommand_name());
        let api_matches = matches.subcommand_matches(api::SUBCOMMAND_NAME).unwrap();
        let api_opts = api::ApiClapOptions::new(&api_matches);
        assert_eq!(api_opts.method, "get");
        assert_eq!(api_opts.path, "endpoint");
        assert_eq!(api_opts.data, "\"name\":\"n\",\"value\":\"d\"");
        assert_eq!(api_opts.server, "death-star");
        assert_eq!(api_opts.api_port, "9999");
        assert_eq!(api_opts.ent, "starwars");
        assert_eq!(api_opts.user, "vader");
    }

    #[test]
    fn test_clap_review_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "review", "--auto-bump",
                                           "--no-open", "--edit", "-f", "custom-pipe"]);
        assert_eq!(Some("review"), matches.subcommand_name());
        let review_matches = matches.subcommand_matches(review::SUBCOMMAND_NAME).unwrap();
        let review_opts = review::ReviewClapOptions::new(&review_matches);
        assert_eq!(review_opts.pipeline, "custom-pipe");
        assert_eq!(review_opts.no_open, true);
        assert_eq!(review_opts.auto_bump, true);
        assert_eq!(review_opts.edit, true);
    }

    #[test]
    fn test_clap_checkout_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "checkout", "change_the_force",
                                           "-P", "p4tchs3t", "-f", "custom-pipe"]);
        assert_eq!(Some("checkout"), matches.subcommand_name());
        let checkout_matches = matches.subcommand_matches(checkout::SUBCOMMAND_NAME).unwrap();
        let checkout_opts = checkout::CheckoutClapOptions::new(&checkout_matches);
        assert_eq!(checkout_opts.pipeline, "custom-pipe");
        assert_eq!(checkout_opts.change, "change_the_force");
        assert_eq!(checkout_opts.patchset, "p4tchs3t");
    }

    #[test]
    fn test_clap_clone_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "clone", "minecraft",
                                           "-e", "world", "-o", "coolest", "-u",
                                           "dummy", "-s", "m.craft.com", "-g",
                                           "ssh://another.world.com:123/awesome"]);
        assert_eq!(Some("clone"), matches.subcommand_name());
        let clone_matches = matches.subcommand_matches(clone::SUBCOMMAND_NAME).unwrap();
        let clone_opts = clone::CloneClapOptions::new(&clone_matches);
        assert_eq!(clone_opts.project, "minecraft");
        assert_eq!(clone_opts.user, "dummy");
        assert_eq!(clone_opts.server, "m.craft.com");
        assert_eq!(clone_opts.ent, "world");
        assert_eq!(clone_opts.org, "coolest");
        assert_eq!(clone_opts.git_url, "ssh://another.world.com:123/awesome");
    }

    #[test]
    fn test_clap_diff_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "diff", "change-me", "-l",
                                           "-P", "p4tchs3t", "-f", "coolest"]);
        assert_eq!(Some("diff"), matches.subcommand_name());
        let diff_matches = matches.subcommand_matches(diff::SUBCOMMAND_NAME).unwrap();
        let diff_opts = diff::DiffClapOptions::new(&diff_matches);
        assert_eq!(diff_opts.change, "change-me");
        assert_eq!(diff_opts.patchset, "p4tchs3t");
        assert_eq!(diff_opts.pipeline, "coolest");
        assert_eq!(diff_opts.local, true);
    }

    #[test]
    fn test_clap_init_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let init_cmd = vec!["delivery", "init", "-l", "-p", "frijol", "-u", "concha",
                        "-s", "cocina.central.com", "-e", "mexicana", "-o", "oaxaca",
                        "-f", "postres", "-c", "receta.json", "--generator", "/original",
                        "--github", "git-mx", "--bitbucket", "bit-mx", "-r", "antojitos",
                        "--no-verify-ssl", "--skip-build-cookbook", "-n"];
        let matches = app.get_matches_from(init_cmd);
        assert_eq!(Some("init"), matches.subcommand_name());
        let init_matches = matches.subcommand_matches(init::SUBCOMMAND_NAME).unwrap();
        let init_opts = init::InitClapOptions::new(&init_matches);
        assert_eq!(init_opts.pipeline, "postres");
        assert_eq!(init_opts.user, "concha");
        assert_eq!(init_opts.server, "cocina.central.com");
        assert_eq!(init_opts.ent, "mexicana");
        assert_eq!(init_opts.org, "oaxaca");
        assert_eq!(init_opts.project, "frijol");
        assert_eq!(init_opts.config_json, "receta.json");
        assert_eq!(init_opts.generator, "/original");
        assert_eq!(init_opts.github_org_name, "git-mx");
        assert_eq!(init_opts.bitbucket_project_key, "bit-mx");
        assert_eq!(init_opts.repo_name, "antojitos");
        assert_eq!(init_opts.pipeline, "postres");
        assert_eq!(init_opts.no_v_ssl, true);
        assert_eq!(init_opts.no_open, true);
        assert_eq!(init_opts.skip_build_cookbook, true);
        assert_eq!(init_opts.local, true);
    }

    #[test]
    fn test_clap_job_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let job_cmd = vec!["delivery", "job", "anime", "ninja", "-C", "rasengan",
                        "-u", "naruto", "-s", "manga.com", "-e", "shippuden", "-o",
                        "akatsuki", "-f", "sharingan", "-j", "/path", "-p", "uchiha",
                        "-P", "latest", "--change-id", "super-cool-id", "-g", "powerful-url",
                        "-S", "SHA", "-b", "evil", "--skip-default", "-l", "--docker", "uzumaki"];
        let matches = app.get_matches_from(job_cmd);
        assert_eq!(Some("job"), matches.subcommand_name());
        let job_matches = matches.subcommand_matches(job::SUBCOMMAND_NAME).unwrap();
        let job_opts = job::JobClapOptions::new(&job_matches);
        assert_eq!(job_opts.pipeline, "sharingan");
        assert_eq!(job_opts.stage, "anime");
        assert_eq!(job_opts.phases, "ninja");
        assert_eq!(job_opts.user, "naruto");
        assert_eq!(job_opts.server, "manga.com");
        assert_eq!(job_opts.change, "rasengan");
        assert_eq!(job_opts.ent, "shippuden");
        assert_eq!(job_opts.org, "akatsuki");
        assert_eq!(job_opts.job_root, "/path");
        assert_eq!(job_opts.project, "uchiha");
        assert_eq!(job_opts.patchset, "latest");
        assert_eq!(job_opts.change_id, "super-cool-id");
        assert_eq!(job_opts.git_url, "powerful-url");
        assert_eq!(job_opts.shasum, "SHA");
        assert_eq!(job_opts.branch, "evil");
        assert_eq!(job_opts.docker_image, "uzumaki");
        assert_eq!(job_opts.local, true);
        assert_eq!(job_opts.skip_default, true);
    }

    #[test]
    fn test_clap_spin_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "spin"]);
        assert_eq!(Some("spin"), matches.subcommand_name());
        let spin_matches = matches.subcommand_matches(spin::SUBCOMMAND_NAME).unwrap();
        let spin_opts = spin::SpinClapOptions::new(&spin_matches);
        assert_eq!(spin_opts.time, 5);
    }

    #[test]
    fn test_clap_token_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "token", "-e", "fellowship",
                                           "-u", "gandalf", "-s", "lord.of.the.rings.com",
                                           "--api-port", "1111", "--verify", "--saml=true"]);
        assert_eq!(Some("token"), matches.subcommand_name());
        let token_matches = matches.subcommand_matches(token::SUBCOMMAND_NAME).unwrap();
        let token_opts = token::TokenClapOptions::new(&token_matches);
        assert_eq!(token_opts.server, "lord.of.the.rings.com");
        assert_eq!(token_opts.port, "1111");
        assert_eq!(token_opts.ent, "fellowship");
        assert_eq!(token_opts.user, "gandalf");
        assert_eq!(token_opts.verify, true);
        assert_eq!(token_opts.saml, Some(true));
    }

    #[test]
    fn test_clap_setup_options() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());
        let app = cli::make_app(&build_version);
        let matches = app.get_matches_from(vec!["delivery", "setup", "-e", "e", "-u", "u",
                                           "-s", "s", "--config-path", "/my/config/cli.toml",
                                           "-f", "p", "-o", "good"]);
        assert_eq!(Some("setup"), matches.subcommand_name());
        let setup_matches = matches.subcommand_matches(setup::SUBCOMMAND_NAME).unwrap();
        let setup_opts = setup::SetupClapOptions::new(&setup_matches);
        assert_eq!(setup_opts.server, "s");
        assert_eq!(setup_opts.org, "good");
        assert_eq!(setup_opts.ent, "e");
        assert_eq!(setup_opts.user, "u");
        assert_eq!(setup_opts.pipeline, "p");
        assert_eq!(setup_opts.path, "/my/config/cli.toml");
    }
}
