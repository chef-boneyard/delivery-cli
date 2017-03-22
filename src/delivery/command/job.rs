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

use git;
use std;
use fips;
use std::env;
use std::process;
use std::io::prelude::*;
use std::path::PathBuf;
use cli::job::JobClapOptions;
use job::workspace::{Workspace, Privilege};
use job::change::Change;
use types::{DeliveryResult, ExitCode};
use errors::{DeliveryError, Kind};
use utils::say::{say, sayln};
use utils::path_join_many::PathJoinMany;
use utils::{self, cwd, privileged_process};
use command::Command;
use config::Config;
use project;

pub struct JobCommand<'n> {
    pub options: &'n JobClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for JobCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        // When `delivery job` runs outside the git_repo, it means we are
        // triggering a job on the Chef Automate Server within the workspace.
        // That means we are not going to be able to retrieve the project_path,
        // if that is the case, it should not initialize any project specific
        // command, like the remote.
        if project::project_path().is_ok() && !self.options.local {
            try!(project::ensure_git_remote_up_to_date(&self.config));
        }

        if !self.options.local {
            try!(fips::setup_and_start_stunnel_if_fips_mode(&self.config, child_processes));
        }

        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");

        let p = try!(self.config.project());
        let s = try!(self.config.server());
        let e = try!(self.config.enterprise());
        let o = try!(self.config.organization());
        let pi = try!(self.config.pipeline());
        say("white", "Starting job for ");
        say("green", &format!("{}", &p));
        say("yellow", &format!(" {}", self.options.stage));
        sayln("magenta", &format!(" {}", self.options.phases));
        let phases: Vec<&str> = self.options.phases.split(" ").collect();
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
        let job_root_path = if self.options.job_root.is_empty() {
            let phase_path: &[&str] = &[&s[..], &e, &o, &p, &pi, self.options.stage, &phase_dir];
            ws_path.join_many(phase_path)
        } else {
            PathBuf::from(self.options.job_root)
        };
        let ws = Workspace::new(&job_root_path);
        sayln("white", &format!("Creating workspace in {}", job_root_path.to_string_lossy()));
        try!(ws.build());
        say("white", "Cloning repository, and merging");
        let mut local_change = false;
        let patch = if self.options.patchset.is_empty() {
            "latest"
        } else {
            self.options.patchset
        };
        let c = if ! self.options.branch.is_empty() {
            say("yellow", &format!(" {}", &self.options.branch));
            String::from(self.options.branch)
        } else if ! self.options.change.is_empty() {
            say("yellow", &format!(" {}", &self.options.change));
            format!("_reviews/{}/{}/{}", pi, self.options.change, patch)
        } else if ! self.options.shasum.is_empty() {
            say("yellow", &format!(" {}", self.options.shasum));
            String::new()
        } else {
            local_change = true;
            let v = try!(git::get_head());
            say("yellow", &format!(" {}", &v));
            v
        };
        say("white", " to ");
        sayln("magenta", &pi);
        let clone_url = if self.options.git_url.is_empty() {
            if local_change {
                cwd().into_os_string().to_string_lossy().into_owned()
            } else {
                try!(self.config.delivery_git_ssh_url())
            }
        } else {
            String::from(self.options.git_url)
        };
        try!(ws.setup_repo_for_change(&clone_url, &c, &pi, self.options.shasum));
        sayln("white", "Configuring the job");
        // This can be optimized out, almost certainly
        try!(utils::remove_recursive(&ws.chef.join("build_cookbook")));
        let change = Change {
            enterprise: e.to_string(),
            organization: o.to_string(),
            project: p.to_string(),
            pipeline: pi.to_string(),
            stage: self.options.stage.to_string(),
            phase: self.options.phases.to_string(),
            git_url: clone_url.to_string(),
            sha: self.options.shasum.to_string(),
            patchset_branch: c.to_string(),
            change_id: self.options.change_id.to_string(),
            patchset_number: patch.to_string()
        };
        try!(ws.setup_chef_for_job(&self.config, change, &ws_path));
        sayln("white", "Running the job");

        let privilege_drop = if privileged_process() {
            Privilege::Drop
        } else {
            Privilege::NoDrop
        };

        if privileged_process() && !&self.options.skip_default {
            sayln("yellow", "Setting up the builder");
            try!(ws.run_job("default", &Privilege::NoDrop, &local_change));
        }

        let phase_msg = if phases.len() > 1 {
            "phases"
        } else {
            "phase"
        };
        sayln("magenta", &format!("Running {} {}", phase_msg, phases.join(", ")));
        try!(ws.run_job(self.options.phases, &privilege_drop, &local_change));
        Ok(0)
    }
}

pub fn run_docker_job(opts: &JobClapOptions) -> DeliveryResult<ExitCode> {
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

    docker.stdout(process::Stdio::piped());
    docker.stderr(process::Stdio::piped());

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

fn maybe_add_flag_value(cmd: &mut process::Command, flag: &str, value: &str) {
    if !value.is_empty() {
        cmd.arg(flag).arg(value);
    }
}

fn maybe_add_flag(cmd: &mut process::Command, flag: &str, value: &bool) {
    if *value {
        cmd.arg(flag);
    }
}

