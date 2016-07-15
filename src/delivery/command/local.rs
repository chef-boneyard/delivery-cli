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

use cli::local::LocalClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{sayln, say};
use std::process::{Stdio};
use delivery_config::project::ProjectToml;
use project;
use utils;

pub fn run(opts: LocalClapOptions) -> DeliveryResult<ExitCode> {
    sayln("green", "Chef Delivery");
    let project_toml: ProjectToml = try!(
        ProjectToml::load_toml_file(project::project_path())
    );
    let phase_cmd = try!(project_toml.local_phase(opts.phase.clone()));
    say("white", "Running ");
    say("magenta", &format!("{:?}", opts.phase.unwrap()));
    sayln("white", " Phase");
    debug!("Executing command: {}", phase_cmd);
    Ok(exec_command(&phase_cmd))
}

pub fn exec_command(cmd: &str) -> ExitCode {
    // TODO: I just copy paste the old code and modified a little bit
    // so it works but we have to work on UW-75 to make it right!
    // We should maybe create a tempfile to stick the command coming from
    // the config instead of running `chef exec` as the command.
    let mut split_cmd = cmd.split_whitespace();
    let c = split_cmd.next().unwrap();
    let args_vec = split_cmd.collect::<Vec<&str>>();
    let output  = utils::make_command(c)
        .args(&args_vec)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(project::project_path())
        .output()
        .unwrap_or_else(|e| { panic!("Unexpected error: Failed to execute process: {}", e) });

    let return_code = match output.status.code() {
        Some(code) => code,
        _ => 1
    };
    return return_code
}
