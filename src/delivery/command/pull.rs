//
// Copyright:: Copyright (c) 2017 Chef Software, Inc.
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

use std;
use fips;
use git;
use errors;
use cli::pull::PullClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{sayln, say};
use config::Config;
use command::Command;

pub struct PullCommand<'n> {
    pub options: &'n PullClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for PullCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        if self.config.fips.unwrap_or(false) {
            try!(super::verify_and_repair_git_remote(&self.config));
            try!(fips::setup_and_start_stunnel(&self.config, child_processes));
        }
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");
        let verb = if self.options.rebase { "Rebasing" } else { "Merging" };
        sayln("white", &format!("{} local HEAD on remote version of {}",
                                verb, self.options.pipeline)
        );

        match git::git_pull(self.options.pipeline, self.options.rebase) {
            Ok(_) => {
                say("white", &format!("HEAD is now on {}", try!(git::git_current_sha())));
                Ok(0)
            },
            Err(errors::DeliveryError{
                kind: errors::Kind::BranchNotFoundOnDeliveryRemote, ..
            }) => {
                sayln("error", &format!("A pipeline or branch named {} was not found on the delivery remote",
                                      self.options.pipeline)
                );
                Ok(1)
            },
            Err(err) => {
                Err(err)
            }
        }
    }
}
