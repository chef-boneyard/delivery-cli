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
use clap::{App, ArgMatches, SubCommand};
use cli::arguments::{a2_mode_arg, project_specific_args, u_e_s_o_args, value_of};
use cli::Options;
use config::Config;
use fips;
use project;
use types::DeliveryResult;

pub const SUBCOMMAND_NAME: &'static str = "clone";

#[derive(Debug)]
pub struct CloneClapOptions<'n> {
    pub project: &'n str,
    pub user: &'n str,
    pub server: &'n str,
    pub ent: &'n str,
    pub org: &'n str,
    pub git_url: &'n str,
    pub fips: bool,
    pub fips_git_port: &'n str,
    pub fips_custom_cert_filename: &'n str,
    pub a2_mode: Option<bool>,
}
impl<'n> Default for CloneClapOptions<'n> {
    fn default() -> Self {
        CloneClapOptions {
            project: "",
            user: "",
            server: "",
            ent: "",
            org: "",
            git_url: "",
            fips: false,
            fips_git_port: "",
            fips_custom_cert_filename: "",
            a2_mode: None,
        }
    }
}

impl<'n> CloneClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        CloneClapOptions {
            project: value_of(&matches, "project"),
            user: value_of(&matches, "user"),
            server: value_of(&matches, "server"),
            ent: value_of(&matches, "ent"),
            org: value_of(&matches, "org"),
            git_url: value_of(&matches, "git-url"),
            fips: matches.is_present("fips"),
            fips_git_port: value_of(&matches, "fips-git-port"),
            fips_custom_cert_filename: value_of(&matches, "fips-custom-cert-filename"),
            a2_mode: if matches.is_present("a2-mode") {
                Some(true)
            } else {
                None
            },
        }
    }
}

impl<'n> Options for CloneClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let mut new_config = config
            .set_user(&self.user)
            .set_server(&self.server)
            .set_enterprise(&self.ent)
            .set_organization(&self.org)
            .set_project(&self.project)
            .set_a2_mode_if_def(self.a2_mode);

        if new_config.project.is_none() {
            new_config.project = project::project_from_cwd().ok();
        }

        fips::merge_fips_options_and_config(
            self.fips,
            self.fips_git_port,
            self.fips_custom_cert_filename,
            new_config,
        )
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Clone a project repository")
        .args_from_usage(
            "<project> 'Name of project to clone'
            -g --git-url=[url] \
            'Git URL (-u -s -e -o ignored if used)'",
        )
        .args(&u_e_s_o_args())
        .args(&project_specific_args())
        .args(&vec![a2_mode_arg()])
}
