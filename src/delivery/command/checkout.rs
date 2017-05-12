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
use cli::checkout::CheckoutClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{sayln, say};
use config::Config;
use command::Command;

pub struct CheckoutCommand<'n> {
    pub options: &'n CheckoutClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for CheckoutCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        try!(super::verify_and_repair_git_remote(&self.config));
        try!(fips::setup_and_start_stunnel_if_fips_mode(&self.config, child_processes));
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");
        let config_ref = self.config;
        let target = validate!(config_ref, pipeline);
        say("white", "Checking out ");
        say("yellow", self.options.change);
        say("white", " targeted for pipeline ");
        say("magenta", &target);

        let pset = match self.options.patchset {
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
        try!(git::checkout_review(self.options.change, pset, &target));
        Ok(0)
    }
}
