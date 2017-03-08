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
use cli::arguments::{pipeline_arg, patchset_arg,
                     value_of, project_specific_args};
use clap::{App, SubCommand, ArgMatches};
use cli::Options;
use types::DeliveryResult;
use config::Config;

pub const SUBCOMMAND_NAME: &'static str = "diff";

#[derive(Debug)]
pub struct DiffClapOptions<'n> {
    pub change: &'n str,
    pub patchset: &'n str,
    pub pipeline: &'n str,
    pub local: bool,
    pub fips: bool,
    pub fips_git_port: &'n str,
    pub fips_custom_cert_filename: &'n str,
}

impl<'n> Default for DiffClapOptions<'n> {
    fn default() -> Self {
        DiffClapOptions {
            change: "",
            patchset: "",
            pipeline: "master",
            local: false,
            fips: false,
            fips_git_port: "",
            fips_custom_cert_filename: "",
        }
    }
}

impl<'n> DiffClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        DiffClapOptions {
            change: value_of(&matches, "change"),
            patchset: value_of(&matches, "patchset"),
            pipeline: value_of(&matches, "pipeline"),
            local: matches.is_present("local"),
            fips: matches.is_present("fips"),
            fips_git_port: value_of(&matches, "fips-git-port"),
            fips_custom_cert_filename: value_of(&matches, "fips-custom-cert-filename"),
        }
    }
}

impl<'n> Options for DiffClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let mut new_config = config.set_pipeline(&self.pipeline);

        if new_config.project.is_none() {
            new_config.project = project::project_from_cwd().ok();
        }

        fips::merge_fips_options_and_config(self.fips, self.fips_git_port,
                                            self.fips_custom_cert_filename, new_config)
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Display diff for a change")
        .args(&vec![patchset_arg()])
        .args(&pipeline_arg())
        .args_from_usage(
            "<change> 'Name of the feature branch to compare'
            -l --local \
            'Diff against the local branch HEAD'")
        .args(&project_specific_args())
}
