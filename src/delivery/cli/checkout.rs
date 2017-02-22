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
use project;
use fips;
use cli::arguments::{pipeline_arg, patchset_arg, value_of, project_specific_args};
use clap::{App, SubCommand, ArgMatches};
use cli::Options;
use config::Config;
use types::DeliveryResult;

pub const SUBCOMMAND_NAME: &'static str = "checkout";

#[derive(Debug)]
pub struct CheckoutClapOptions<'n> {
    pub pipeline: &'n str,
    pub change: &'n str,
    pub patchset: &'n str,
    pub fips: bool,
    pub fips_git_port: &'n str,
}

impl<'n> Default for CheckoutClapOptions<'n> {
    fn default() -> Self {
        CheckoutClapOptions {
            pipeline: "master",
            change: "",
            patchset: "",
            fips: false,
            fips_git_port: "",
        }
    }
}

impl<'n> CheckoutClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        CheckoutClapOptions {
            pipeline: value_of(&matches, "pipeline"),
            change: value_of(&matches, "change"),
            patchset: value_of(&matches, "patchset"),
            fips: matches.is_present("fips"),
            fips_git_port: value_of(&matches, "fips-git-port"),
        }
    }
}

impl<'n> Options for CheckoutClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let mut new_config = config.set_pipeline(&self.pipeline);

        if new_config.project.is_none() {
            new_config.project = project::project_from_cwd().ok();
        }

        fips::merge_fips_options_and_config(self.fips, self.fips_git_port, new_config)
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Create a local branch tracking an in-progress change")
        .args(&vec![patchset_arg()])
        .args(&pipeline_arg())
        .args_from_usage("<change> 'Name of the feature branch to checkout'")
        .args(&project_specific_args())
}
