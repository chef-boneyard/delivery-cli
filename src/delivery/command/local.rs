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
use command::Command;
use delivery_config::project::{Phase, ProjectToml};
use errors::{DeliveryError, Kind};
use project;
use std::process::Stdio;
use types::{DeliveryResult, ExitCode};
use utils;
use utils::say::{say, sayln};

pub struct LocalCommand<'n> {
    pub options: &'n LocalClapOptions<'n>,
    pub config: &'n ProjectToml,
}

impl<'n> Command for LocalCommand<'n> {
    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");

        // If a Stage was provided, trigger their phases in order
        if let Some(stage) = self.options.stage.clone() {
            say("white", "Running ");
            say("yellow", &format!("{}", stage));
            sayln("white", " Stage");
            for phase in stage.phases().into_iter() {
                match try!(exec_phase(&self.config.clone(), Some(phase))) {
                    0 => continue,
                    exit_code => {
                        return Err(DeliveryError {
                            kind: Kind::PhaseFailed(exit_code),
                            detail: None,
                        })
                    }
                }
            }
            Ok(0)
        } else {
            exec_phase(self.config, self.options.phase.clone())
        }
    }
}

fn exec_phase(project_toml: &ProjectToml, phase: Option<Phase>) -> DeliveryResult<ExitCode> {
    if let Some(phase_cmd) = try!(project_toml.local_phase(phase.clone())) {
        say("white", "Running ");
        say("magenta", &format!("{:?}", phase.unwrap()));
        sayln("white", " Phase");
        debug!("Executing command: {}", phase_cmd);
        exec_command(&phase_cmd)
    } else {
        let p = phase.unwrap();
        sayln(
            "red",
            &format!(
                "Unable to execute an empty phase.\nPlease verify that \
                              your project.toml has a {} phase configured as follows:
                              \n[local_phases]\n{} = \"insert script here\"",
                p, p
            ),
        );
        Ok(1)
    }
}
fn exec_command(cmd: &str) -> DeliveryResult<ExitCode> {
    // TODO: I just copy paste the old code and modified a little bit
    // so it works but we have to work on UW-75 to make it right!
    // We should maybe create a tempfile to stick the command coming from
    // the config instead of running `chef exec` as the command.
    let mut split_cmd = cmd.split_whitespace();
    let c = split_cmd.next().unwrap();
    let args_vec = split_cmd.collect::<Vec<&str>>();
    let output = utils::make_command(c)
        .args(&args_vec)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(try!(project::project_path()))
        .output()
        .unwrap_or_else(|e| panic!("Unexpected error: Failed to execute process: {}", e));

    let return_code = match output.status.code() {
        Some(code) => code,
        _ => 1,
    };
    Ok(return_code)
}
