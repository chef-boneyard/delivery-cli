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

use config::Config;
use git;
use project;
use std;
use types::{DeliveryResult, ExitCode};
use utils;
use utils::cwd;
use utils::say::sayln;

pub mod api;
pub mod checkout;
pub mod clone;
pub mod diff;
pub mod init;
pub mod job;
pub mod local;
pub mod pull;
pub mod review;
pub mod setup;
pub mod status;
pub mod token;

pub trait Command: Sized {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        let _ = child_processes;
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode>;

    fn teardown(&self, child_processes: Vec<std::process::Child>) -> DeliveryResult<()> {
        utils::kill_child_processes(child_processes)
    }
}

// Common functions for Commands
//
// There will be cases where commands might need to share certain actions across
// other commands that requires the user to interact by answering keystrokes. An
// example could be when we detect that the delivery remote needs to be updated
// and we need confirmation from the user to proceed.
//
// Once you have added the method, you can just call it with `super::foo()` from
// within any command. (don't forget to make it public)
pub fn verify_and_repair_git_remote(config: &Config) -> DeliveryResult<()> {
    if !project::git_remote_up_to_date(config)? {
        let p_path = project::project_path()?;
        let c_path = Config::dot_delivery_cli_path(&cwd()).expect("Unable to find cli.toml");
        let git_ssh_url = config.delivery_git_ssh_url()?;
        let msg = &format!(
            "Updating 'delivery' remote with the default configuration \
             loaded from {:?}.\n\tcurrent: {}\n\tupdate:  {}",
            c_path,
            &git::delivery_remote_from_repo(&p_path)?,
            &git_ssh_url
        );
        sayln("yellow", msg);
        try!(git::update_delivery_remote(&git_ssh_url, &p_path));
    }
    Ok(())
}
