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
use cli::arguments::{value_of, project_specific_args};
use clap::{App, SubCommand, ArgMatches};
use cli::Options;
use config::Config;
use types::DeliveryResult;

pub const SUBCOMMAND_NAME: &'static str = "pull";

#[derive(Debug)]
pub struct PullClapOptions<'n> {
    pub pipeline: &'n str,
    pub fips: bool,
    pub fips_git_port: &'n str,
    pub rebase: bool,
}

impl<'n> Default for PullClapOptions<'n> {
    fn default() -> Self {
        PullClapOptions {
            pipeline: "",
            fips: false,
            fips_git_port: "",
            rebase: false,
        }
    }
}

impl<'n> PullClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        PullClapOptions {
            pipeline: value_of(&matches, "pipeline"),
            fips: matches.is_present("fips"),
            fips_git_port: value_of(&matches, "fips-git-port"),
            rebase: matches.is_present("rebase"),
        }
    }
}

impl<'n> Options for PullClapOptions<'n> {
    fn merge_options_and_config(&self, mut config: Config) -> DeliveryResult<Config> {
        if config.project.is_none() {
            config.project = project::project_from_cwd().ok();
        }

        fips::merge_fips_options_and_config(self.fips, self.fips_git_port, config)
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Retrieve a pipeline from Automate and merge your current pipeline on it")
        .args_from_usage(
            "<pipeline> 'Name of the remote pipeline on the Automate server to retrieve (can also be any git ref such as a branch)'
            --rebase 'Performs a rebase on the pipeline retrieved from Automate server instead of a merge'"
        )
        .args(&project_specific_args())
}
