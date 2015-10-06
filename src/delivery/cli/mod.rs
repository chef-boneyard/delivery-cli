use std::env;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::error::Error;
use std::path::PathBuf;
use std::error;
use std::path::Path;
use std::io::prelude::*;

use utils::{self, privileged_process};

use utils::say::{say, sayln};
use utils::mkdir_recursive;
use errors::{DeliveryError, Kind};
use config::Config;
use delivery_config::DeliveryConfig;
use git::{self, ReviewResult};
use job::change::Change;
use job::workspace::{Workspace, Privilege};
use utils::path_join_many::PathJoinMany;
use token;
use http::{self, APIClient};
use hyper::status::StatusCode;
use project;
use clap::{Arg, App, SubCommand, ArgMatches};

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
        fn $fn_name<'a>() -> Arg<'a, 'a, 'a, 'a, 'a, 'a> {
            Arg::from_usage($usage)
        }
    )
}

fn u_e_s_o_args<'a>() -> Vec<Arg<'a, 'a, 'a, 'a, 'a, 'a>> {
    make_arg_vec![
        "-u --user=[user] 'User name for Delivery authentication'",
        "-e --ent=[enterprise] 'The enterprise in which the project lives'",
        "-o --org=[org] 'The organization in which the project lives'",
        "-s --server=[server] 'The Delivery server address'"]
}

fn_arg!(for_arg,
       "-f --for=[pipeline] 'Target pipeline for change (default: master)'");

fn_arg!(patchset_arg,
       "-P --patchset=[patchset] 'A patchset number (default: latest)'");

fn_arg!(project_arg,
       "-p --project=[project] 'The project name'");

fn_arg!(config_path_arg,
        "--config-path=[dir] 'Directory to read/write your config file \
         (cli.toml)'");

fn_arg!(local_arg, "-l --local 'Operate without a Delivery server'");

fn_arg!(no_open_arg, "-n --no-open 'Do not open the change in a browser'");


fn_arg!(no_spinner_arg, "--no-spinner 'Disable the spinner'");

macro_rules! validate {
    ($config:ident, $value:ident) => (
        try!($config.$value());
    )
}

pub fn run() {
    let build_version = format!("{} {}", version(), build_git_sha());

    let app = make_app(&build_version);
    let matches = app.get_matches();

    let cmd_result = match matches.subcommand_name() {
        Some("api") => {
            let matches = matches.subcommand_matches("api").unwrap();
            clap_api_req(matches)
        },
        Some("checkout") => {
            let matches = matches.subcommand_matches("checkout").unwrap();
            clap_checkout(matches)
        },
        Some("clone") => {
            let matches = matches.subcommand_matches("clone").unwrap();
            clap_clone(matches)
        },
        Some("diff") => {
            let matches = matches.subcommand_matches("diff").unwrap();
            clap_diff(matches)
        },
        Some("init") => {
            let matches = matches.subcommand_matches("init").unwrap();
            clap_init(matches)
        },
        Some("job") => {
            let matches = matches.subcommand_matches("job").unwrap();
            clap_job(matches)
        },
        Some("review") => {
            let matches = matches.subcommand_matches("review").unwrap();
            clap_review(matches)
        },
        Some("setup") => {
            let matches = matches.subcommand_matches("setup").unwrap();
            clap_setup(matches)
        },
        Some("token") => {
            let matches = matches.subcommand_matches("token").unwrap();
            clap_token(matches)
        },
        _ => {
            // ownership issue with use of above defined app
            // so for now...
            let a = make_app(&build_version);
            a.print_help().ok().expect("failed to write help to stdout");
            sayln("red", "missing subcommand");
            process::exit(1);
        }
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}

fn make_app<'a>(version: &'a str) -> App<'a, 'a, 'a, 'a, 'a, 'a> {
    App::new("delivery")
        .version(version)
        .subcommand(SubCommand::with_name("review")
                    .about("Submit current branch for review")
                    // NOTE: in the future, we can add extensive
                    // sub-command specific help via an include file
                    // like this:
                    // .after_help(include!("../help/create-change.txt"))
                    .args(vec![for_arg(), no_open_arg()])
                    .args_from_usage(
                        "-e --edit 'Edit change title and description'"))
        .subcommand(SubCommand::with_name("clone")
                    .about("Clone a project repository")
                    .args_from_usage(
                        "<project> 'Name of project to clone'
                        -g --git-url=[url] \
                        'Git URL (-u -s -e -o ignored if used)'")
                    .args(u_e_s_o_args()))
        .subcommand(SubCommand::with_name("checkout")
                    .about("Create a local branch tracking an in-progress change")
                    .args(vec![for_arg(), patchset_arg()])
                    .args_from_usage(
                        "<change> 'Name of the feature branch to checkout'"))
        .subcommand(SubCommand::with_name("diff")
                    .about("Display diff for a change")
                    .args(vec![for_arg(), patchset_arg()])
                    .args_from_usage(
                        "<change> 'Name of the feature branch to compare'
                        -l --local \
                        'Diff against the local branch HEAD'"))
        .subcommand(SubCommand::with_name("init")
                    .about("Add delivery remote to this git repo \
                            (and lot's more!)")
                    .args(vec![for_arg(), config_path_arg(), no_open_arg(),
                               project_arg(), local_arg()])
                    .args_from_usage(
                        "--skip-build-cookbook 'Do not create a build cookbook'")
                    .args(u_e_s_o_args()))
        .subcommand(SubCommand::with_name("setup")
                    .about("Write a config file capturing specified options")
                    .args(vec![for_arg(), config_path_arg()])
                    .args(u_e_s_o_args()))
        .subcommand(SubCommand::with_name("job")
                    .about("Run one or more phase jobs")
                    .args(vec![patchset_arg(), project_arg(), for_arg(),
                               local_arg(), no_spinner_arg()])
                    .args(make_arg_vec![
                        "-j --job-root=[root] 'Path to the job root'",
                        "-g --git-url=[url] 'Git URL (-u -s -e -o ignored if used)'",
                        "-C --change=[change] 'Feature branch name'",
                        "-b --branch=[branch] 'Branch to merge'",
                        "-S --shasum=[gitsha] 'Git SHA of change'",
                        "--change-id=[id] 'The change ID'",
                        "--skip-default 'skip default'",
                        "--docker=[image] 'Docker image'"])
                    .args_from_usage("<stage> 'Stage for the run'
                                      <phases> 'One or more phases'")
                    .args(u_e_s_o_args()))
        .subcommand(SubCommand::with_name("api")
                    .about("Helper to call Delivery's HTTP API")
                    .args(vec![config_path_arg()])
                    .args_from_usage(
                        "<method> 'HTTP method for the request'
                         <path> 'Path for rqeuest URL'
                         --api-port=[port] 'Port for Delivery server'
                         -d --data=[data] 'Data to send for PUT/POST request'")
                    .args(u_e_s_o_args()))
        .subcommand(SubCommand::with_name("token")
                    .about("Create a local API token")
                    .args(make_arg_vec![
                        "-u --user=[user] 'User name for Delivery authentication'",
                        "-e --ent=[enterprise] 'The enterprise in which the project lives'",
                        "-s --server=[server] 'The Delivery server address'"])
                    .args_from_usage(
                        "--api-port=[port] 'Port for Delivery server'"))
}

fn cwd() -> PathBuf {
    env::current_dir().unwrap()
}

fn exit_with(e: DeliveryError, i: isize) {
    sayln("red", e.description());
    match e.detail() {
        Some(deets) => sayln("red", &deets),
        None => {}
    }
    let x = i as i32;
    process::exit(x)
}

fn load_config(path: &PathBuf) -> Result<Config, DeliveryError> {
    say("white", "Loading configuration from ");
    let msg = format!("{}", path.display());
    sayln("yellow", &msg);
    let config = try!(Config::load_config(&cwd()));
    Ok(config)
}

fn clap_setup(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let user = value_of(&matches, "user");
    let server = value_of(&matches, "server");
    let ent = value_of(&matches, "enterprise");
    let org = value_of(&matches, "org");
    let path = value_of(&matches, "dir");
    let pipeline = value_of(&matches, "pipeline");
    setup(user, server, ent, org, path, pipeline)
}

fn setup(user: &str, server: &str, ent: &str,
         org: &str, path: &str, pipeline: &str) -> Result<(), DeliveryError> {
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

fn clap_init(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let user = value_of(&matches, "user");
    let server = value_of(&matches, "server");
    let ent = value_of(&matches, "enterprise");
    let org = value_of(&matches, "org");
    let proj = value_of(&matches, "project");
    let no_open = matches.is_present("no-open");
    let skip_build_cookbook = matches.is_present("skip-build-cookbook");
    let local = matches.is_present("local");
    init(user, server, ent, org, proj, &no_open, &skip_build_cookbook, &local)
}

fn init(user: &str, server: &str, ent: &str, org: &str, proj: &str,
        no_open: &bool,skip_build_cookbook: &bool,
        local: &bool) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");

    let mut config = try!(load_config(&cwd()));
    let final_proj = try!(project_or_from_cwd(proj));
    config = config.set_user(user)
        .set_server(server)
        .set_enterprise(ent)
        .set_organization(org)
        .set_project(&final_proj);

    let cwd = try!(env::current_dir());
    if !local {
        try!(project::import(&config, &cwd));
    }

    // we want to generate the build cookbook by default. let the user
    // decide to skip if they don't want one.
    if ! *skip_build_cookbook {

        sayln("white", "Generating build cookbook skeleton");

        let pcb_dir = match utils::home_dir(&[".delivery/cache/generator-cookbooks/pcb"]) {
            Ok(p) => p,
            Err(e) => return Err(e)
        };

        if pcb_dir.exists() {
            sayln("yellow", "Cached copy of build cookbook generator exists; skipping git clone.");
        } else {
            sayln("white", &format!("Cloning build cookbook generator dir {:#?}", pcb_dir));

            try!(git::clone(&pcb_dir.to_string_lossy(),
                            "https://github.com/chef-cookbooks/pcb"));
        }

        // Generate the cookbook
        let dot_delivery = Path::new(".delivery");
        try!(mkdir_recursive(dot_delivery));
        let mut gen = utils::make_command("chef");
        gen.arg("generate")
            .arg("cookbook")
            .arg(".delivery/build-cookbook")
            .arg("-g")
            .arg(pcb_dir);

        match gen.output() {
            Ok(o) => o,
            Err(e) => return Err(DeliveryError {
                                     kind: Kind::FailedToExecute,
                detail: Some(format!("failed to execute chef generate: {}", error::Error::description(&e)))})
        };

        let msg = format!("PCB generate: {:#?}", gen);
        sayln("green", &msg);

        sayln("white", "Git add and commit of build-cookbook");
        try!(git::git_command(&["add", ".delivery/build-cookbook"], &cwd));
        try!(git::git_command(&["commit", "-m", "Add Delivery build cookbook"], &cwd));
    }

    // now to adding the .delivery/config.json, this uses our
    // generated build cookbook always, so we no longer need a project
    // type.
    try!(DeliveryConfig::init(&cwd));

    if !local {
        // if we got here, we've checked out a feature branch, added a
        // config file, added a build cookbook, and made appropriate local
        // commit(s).
        // Let's create the review!
        try!(review("master", no_open, &false));
    }
    Ok(())
}

fn clap_review(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let pipeline = value_of(&matches, "pipeline");
    let no_open = matches.is_present("no-open");
    let edit = matches.is_present("edit");
    review(pipeline, &no_open, &edit)
}

fn review(for_pipeline: &str,
          no_open: &bool, edit: &bool) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_pipeline(for_pipeline);
    let target = validate!(config, pipeline);
    // validate the delivery config file
    // TODO: same as elsewhere in the code, we should get the project's root
    // (instead of simply cwd), e.g. by looking for the .git dir?
    let cwd = try!(env::current_dir());
    try!(DeliveryConfig::validate_config_file(&cwd));

    say("white", "Review for change ");
    let head = try!(git::get_head());
    if &target == &head {
        return Err(DeliveryError{ kind: Kind::CannotReviewSameBranch, detail: None })
    }
    say("yellow", &head);
    say("white", " targeted for pipeline ");
    sayln("magenta", &target);
    let review = try!(git::git_push_review(&head, &target));
    if *edit {
        let project = try!(project_from_cwd());
        config = config.set_pipeline(for_pipeline)
            .set_project(&project);

        try!(edit_change(&config, &review));
    }
    handle_review_result(&review, no_open)
}

fn edit_change(config: &Config,
               review: &ReviewResult) -> Result<(), DeliveryError> {
    let proj = try!(config.project());
    match review.change_id {
        Some(ref change_id) => {
            let change0 = try!(http::change::get(&config, &change_id));
            let text0 = format!("{}\n\n{}\n",
                                change0.title, change0.description);
            let text1 = try!(utils::open::edit_str(&proj, &text0));
            let change1 = try!(http::change::Description::parse_text(&text1));
            Ok(try!(http::change::set(&config, &change_id, &change1)))
        },
        None => Ok(())
    }
}

fn handle_review_result(review: &ReviewResult,
                        no_open: &bool) -> Result<(), DeliveryError> {
    for line in review.messages.iter() {
        sayln("white", line);
    }
    match review.url {
        Some(ref url) => {
            sayln("magenta", &url);
            if !no_open {
                try!(utils::open::item(&url));
            }
        },
        None => {}
    };
    Ok(())
}

fn clap_checkout(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let change = matches.value_of("change").unwrap();
    let patchset = value_of(&matches, "patchset");
    let pipeline = value_of(&matches, "pipeline");
    checkout(change, patchset, pipeline)
}

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

fn clap_diff(matches: &ArgMatches) ->  Result<(), DeliveryError> {
    let change = matches.value_of("change").unwrap();
    let patchset = value_of(&matches, "patchset");
    let pipeline = value_of(&matches, "pipeline");
    let local = matches.is_present("local");
    diff(change, patchset, pipeline, &local)
}

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

fn clap_clone(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let project = matches.value_of("project").unwrap();
    let user = value_of(&matches, "user");
    let server = value_of(&matches, "server");
    let ent = value_of(&matches, "enterprise");
    let org = value_of(&matches, "org");
    let git_url = value_of(&matches, "url");
    clone(project, user, server, ent, org, git_url)
}

fn clone(project: &str, user: &str, server: &str, ent: &str, org: &str, git_url: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_user(user)
        .set_server(server)
        .set_enterprise(ent)
        .set_organization(org)
        .set_project(project);
    say("white", "Cloning ");
    let delivery_url = try!(config.delivery_git_ssh_url());
    let clone_url = if git_url.is_empty() {
        delivery_url.clone()
    } else {
        String::from(git_url)
    };
    say("yellow", &clone_url);
    say("white", " to ");
    sayln("magenta", &format!("{}", project));
    try!(git::clone(project, &clone_url));
    let project_root = cwd().join(project);
    try!(git::config_repo(&delivery_url,
                          &project_root));
    Ok(())
}

fn clap_job(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let stage = matches.value_of("stage").unwrap();
    let phases = matches.value_of("phases").unwrap();

    let change = value_of(&matches, "change");
    let pipeline = value_of(&matches, "pipeline");
    let job_root = value_of(&matches, "root");
    let proj = value_of(&matches, "project");

    let user = value_of(&matches, "user");
    let server = value_of(&matches, "server");
    let ent = value_of(&matches, "enterprise");
    let org = value_of(&matches, "org");

    let patchset = value_of(&matches, "patchset");
    let change_id = value_of(&matches, "id");
    let git_url = value_of(&matches, "url");
    let shasum = value_of(&matches, "gitsha");
    let branch = value_of(&matches, "branch");

    let skip_default = matches.is_present("skip-default");
    let local = matches.is_present("local");
    let docker_image = value_of(&matches, "image");

    job(stage, phases, change, pipeline, job_root,
        proj, user, server, ent, org, patchset,
        change_id, git_url, shasum, branch,
        &skip_default, &local, docker_image)
}

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
       shasum: &str,
       branch: &str,
       skip_default: &bool,
       local: &bool,
       docker_image: &str) -> Result<(), DeliveryError>
{
    sayln("green", "Chef Delivery");
    if !docker_image.is_empty() {
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
            .arg(docker_image)
            .arg("delivery").arg("job").arg(stage).arg(phase);

        let flags_with_values = vec![("--change", change),
                                     ("--for", pipeline),
                                     ("--job-root", job_root),
                                     ("--project", project),
                                     ("--user", user),
                                     ("--server", server),
                                     ("--ent", ent),
                                     ("--org", org),
                                     ("--patchset", patchset),
                                     ("--change_id", change_id),
                                     ("--git-url", git_url),
                                     ("--shasum", shasum),
                                     ("--branch", branch)];

        for (flag, value) in flags_with_values {
            maybe_add_flag_value(&mut docker, flag, value);
        }

        let flags = vec![("--skip-default", skip_default),
                         ("--local", local)];

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
        return Ok(());
    }

    let mut config = try!(load_config(&cwd()));
    config = if project.is_empty() {
        let filename = String::from(cwd().file_name().unwrap().to_str().unwrap());
        config.set_project(&filename)
    } else {
        config.set_project(project)
    };

    config = config.set_pipeline(pipeline)
        .set_user(with_default(user, "you", local))
        .set_server(with_default(server, "localhost", local))
        .set_enterprise(with_default(ent, "local", local))
        .set_organization(with_default(org, "workstation", local));
    let p = try!(config.project());
    let s = try!(config.server());
    let e = try!(config.enterprise());
    let o = try!(config.organization());
    let pi = try!(config.pipeline());
    say("white", "Starting job for ");
    say("green", &format!("{}", &p));
    say("yellow", &format!(" {}", stage));
    sayln("magenta", &format!(" {}", phase));
    let phases: Vec<&str> = phase.split(" ").collect();
    let phase_dir = phases.join("-");
    let ws_path = match env::home_dir() {
        Some(path) => if privileged_process() {
                          PathBuf::from(path)
                      } else {
                          PathBuf::from(path).join_many(&[".delivery"])
                      },
        None => return Err(DeliveryError{ kind: Kind::NoHomedir, detail: None })
    };
    debug!("Workspace Path: {}", ws_path.display());
    let job_root_path = if job_root.is_empty() {
        let phase_path: &[&str] = &[&s[..], &e, &o, &p, &pi, stage, &phase_dir];
        ws_path.join_many(phase_path)
    } else {
        PathBuf::from(job_root)
    };
    let ws = Workspace::new(&job_root_path);
    sayln("white", &format!("Creating workspace in {}", job_root_path.to_string_lossy()));
    try!(ws.build());
    say("white", "Cloning repository, and merging");
    let mut local_change = false;
    let patch = if patchset.is_empty() { "latest" } else { patchset };
    let c = if ! branch.is_empty() {
        say("yellow", &format!(" {}", &branch));
        String::from(branch)
    } else if ! change.is_empty() {
        say("yellow", &format!(" {}", &change));
        format!("_reviews/{}/{}/{}", pi, change, patch)
    } else if ! shasum.is_empty() {
        say("yellow", &format!(" {}", shasum));
        String::new()
    } else {
        local_change = true;
        let v = try!(git::get_head());
        say("yellow", &format!(" {}", &v));
        v
    };
    say("white", " to ");
    sayln("magenta", &pi);
    let clone_url = if git_url.is_empty() {
        if local_change {
            cwd().into_os_string().to_string_lossy().into_owned()
        } else {
            try!(config.delivery_git_ssh_url())
        }
    } else {
        String::from(git_url)
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
    try!(ws.setup_chef_for_job(&config, change, &ws_path));
    sayln("white", "Running the job");

    let privilege_drop = if privileged_process() {
        Privilege::Drop
    } else {
        Privilege::NoDrop
    };

    if privileged_process() && !skip_default {
        sayln("yellow", "Setting up the builder");
        try!(ws.run_job("default", &Privilege::NoDrop, &local_change));
    }

    let phase_msg = if phases.len() > 1 {
        "phases"
    } else {
        "phase"
    };
    sayln("magenta", &format!("Running {} {}", phase_msg, phases.join(", ")));
    try!(ws.run_job(phase, &privilege_drop, &local_change));
    Ok(())
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

fn clap_token(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let server = value_of(&matches, "server");
    let port = value_of(&matches, "port");
    let ent = value_of(&matches, "enterprise");
    let user = value_of(&matches, "user");
    api_token(server, port, ent, user)
}

fn api_token(server: &str, port: &str, ent: &str,
             user: &str) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    config = config.set_server(server)
        .set_api_port(port)
        .set_enterprise(ent)
        .set_user(user);
    try!(token::TokenStore::request_token(&config));
    Ok(())
}

fn version() -> String {
    let epoch = "1970-01-01T00:00:00Z";
    let build_time = option_env!("DELIV_CLI_TIME").unwrap_or(epoch);
    format!("{}", build_time)
}

fn build_git_sha() -> String {
    let sha = option_env!("DELIV_CLI_GIT_SHA").unwrap_or("0000");
    format!("({})", sha)
}

fn clap_api_req(matches: &ArgMatches) -> Result<(), DeliveryError> {
    let method = matches.value_of("method").unwrap();
    let path = matches.value_of("path").unwrap();
    let data = value_of(&matches, "data");

    let server = value_of(&matches, "server");
    let api_port = value_of(&matches, "port");
    let ent = value_of(&matches, "enterprise");
    let user = value_of(&matches, "user");
    api_req(method, path, data, server, api_port, ent, user)
}

fn api_req(method: &str, path: &str, data: &str,
           server: &str, api_port: &str, ent: &str, user: &str) -> Result<(), DeliveryError> {
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_user(user)
        .set_server(server)
        .set_api_port(api_port)
        .set_enterprise(ent);
    let client = try!(APIClient::from_config(&config));
    let mut result = match method {
        "get" => try!(client.get(path)),
        "post" => try!(client.post(path, data)),
        "put" => try!(client.put(path, data)),
        "delete" => try!(client.delete(path)),
        _ => return Err(DeliveryError{ kind: Kind::UnsupportedHttpMethod,
                                       detail: None })
    };
    match result.status {
        StatusCode::NoContent => {},
        _ => {
            let pretty_json = try!(APIClient::extract_pretty_json(&mut result));
            println!("{}", pretty_json);
        }
    };
    Ok(())
}

fn project_from_cwd() -> Result<String, DeliveryError> {
    let cwd = try!(env::current_dir());
    Ok(cwd.file_name().unwrap().to_str().unwrap().to_string())
}

fn project_or_from_cwd(proj: &str) -> Result<String, DeliveryError> {
    if proj.is_empty() {
        project_from_cwd()
    } else {
        Ok(proj.to_string())
    }
}

fn value_of<'a>(matches: &'a ArgMatches, key: &str) -> &'a str {
    matches.value_of(key).unwrap_or("")
}
