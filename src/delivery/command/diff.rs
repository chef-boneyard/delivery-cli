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

use std;
use fips;
use git;
use cli::diff::DiffClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{say, sayln};
use command::Command;
use config::Config;

pub struct DiffCommand<'n> {
    pub options: &'n DiffClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for DiffCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        if !self.options.local {
            try!(super::verify_and_repair_git_remote(&self.config));
            try!(fips::setup_and_start_stunnel_if_fips_mode(&self.config, child_processes));
        }

        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");
        let config_ref = self.config;
        let target = validate!(config_ref, pipeline);
        say("white", "Showing diff for ");
        say("yellow", self.options.change);
        say("white", " targeted for pipeline ");
        say("magenta", &target);

        if self.options.patchset == "latest" {
            sayln("white", " latest patchset");
        } else {
            say("white", " at patchset ");
            sayln("yellow", self.options.patchset);
        }
        try!(git::diff(self.options.change, self.options.patchset, &target, &self.options.local));
        Ok(0)
    }
}
