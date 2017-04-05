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
use cli::arguments::{pipeline_arg, no_open_arg,
                     value_of, auto_bump, project_specific_args};
use clap::{App, SubCommand, ArgMatches};
use config::Config;
use types::DeliveryResult;
use cli::Options;

pub const SUBCOMMAND_NAME: &'static str = "review";

#[derive(Debug)]
pub struct ReviewClapOptions<'n> {
    pub pipeline: &'n str,
    pub no_open: bool,
    pub auto_bump: bool,
    pub edit: bool,
    pub fips: bool,
    pub fips_git_port: &'n str,
    pub fips_custom_cert_filename: &'n str,
    pub user: &'n str,
}
impl<'n> Default for ReviewClapOptions<'n> {
    fn default() -> Self {
        ReviewClapOptions {
            pipeline: "master",
            no_open: false,
            auto_bump: false,
            edit: false,
            fips: false,
            fips_git_port: "",
            fips_custom_cert_filename: "",
            user: "",
        }
    }
}

impl<'n> ReviewClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        ReviewClapOptions {
            pipeline: value_of(&matches, "pipeline"),
            no_open: matches.is_present("no-open"),
            auto_bump: matches.is_present("auto-bump"),
            edit: matches.is_present("edit"),
            fips: matches.is_present("fips"),
            fips_git_port: value_of(&matches, "fips-git-port"),
            fips_custom_cert_filename: value_of(&matches, "fips-custom-cert-filename"),
            user: value_of(&matches, "user"),
        }
    }
}

impl<'n> Options for ReviewClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let mut new_config = config.set_pipeline(&self.pipeline)
            .set_user(&self.user);

        if new_config.auto_bump.is_none() {
            new_config.auto_bump = Some(self.auto_bump);
        }

        if new_config.project.is_none() {
            new_config.project = project::project_from_cwd().ok();
        }

        fips::merge_fips_options_and_config(self.fips, self.fips_git_port,
                                            self.fips_custom_cert_filename, new_config)
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Submit current branch for review")
        .args(&vec![no_open_arg(), auto_bump()])
        .args_from_usage("-e --edit 'Edit change title and description'")
        .args(&pipeline_arg())
        .args(&project_specific_args())
        .args_from_usage("-u --user=[user] 'Automate user name for authentication'")
}
