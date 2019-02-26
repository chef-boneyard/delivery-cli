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

use clap::{App, Arg, ArgMatches, SubCommand};
use cli::arguments::{
    a2_mode_arg, local_arg, patchset_arg, pipeline_arg, project_arg, project_specific_args,
    u_e_s_o_args, value_of,
};
use cli::Options;
use config::Config;
use fips;
use project;
use types::DeliveryResult;

pub const SUBCOMMAND_NAME: &'static str = "job";

#[derive(Debug)]
pub struct JobClapOptions<'n> {
    pub stage: &'n str,
    pub phases: &'n str,
    pub change: &'n str,
    pub pipeline: &'n str,
    pub job_root: &'n str,
    pub project: &'n str,
    pub user: &'n str,
    pub server: &'n str,
    pub ent: &'n str,
    pub org: &'n str,
    pub patchset: &'n str,
    pub change_id: &'n str,
    pub git_url: &'n str,
    pub shasum: &'n str,
    pub branch: &'n str,
    pub skip_default: bool,
    pub local: bool,
    pub docker_image: &'n str,
    pub fips: bool,
    pub fips_git_port: &'n str,
    pub fips_custom_cert_filename: &'n str,
    pub a2_mode: Option<bool>,
}

impl<'n> Default for JobClapOptions<'n> {
    fn default() -> Self {
        JobClapOptions {
            stage: "",
            phases: "",
            change: "",
            pipeline: "master",
            job_root: "",
            project: "",
            user: "",
            server: "",
            ent: "",
            org: "",
            patchset: "",
            change_id: "",
            git_url: "",
            shasum: "",
            branch: "",
            skip_default: false,
            local: false,
            docker_image: "",
            fips: false,
            fips_git_port: "",
            fips_custom_cert_filename: "",
            a2_mode: None,
        }
    }
}

impl<'n> JobClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        JobClapOptions {
            stage: value_of(&matches, "stage"),
            phases: value_of(matches, "phases"),
            change: value_of(&matches, "change"),
            pipeline: value_of(&matches, "pipeline"),
            job_root: value_of(&matches, "job-root"),
            project: value_of(&matches, "project"),
            user: value_of(&matches, "user"),
            server: value_of(&matches, "server"),
            ent: value_of(&matches, "ent"),
            org: value_of(&matches, "org"),
            patchset: value_of(&matches, "patchset"),
            change_id: value_of(&matches, "change-id"),
            git_url: value_of(&matches, "git-url"),
            shasum: value_of(&matches, "shasum"),
            branch: value_of(&matches, "branch"),
            skip_default: matches.is_present("skip-default"),
            local: matches.is_present("local"),
            docker_image: value_of(&matches, "docker"),
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

impl<'n> Options for JobClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let project = try!(project::project_or_from_cwd(&self.project));

        let mut new_config = config
            .set_pipeline(&self.pipeline)
            .set_user(with_default(&self.user, "you", &&self.local))
            .set_server(with_default(&self.server, "localhost", &&self.local))
            .set_enterprise(with_default(&self.ent, "local", &&self.local))
            .set_organization(with_default(&self.org, "workstation", &&self.local))
            .set_project(&project)
            .set_a2_mode_if_def(self.a2_mode);

        // A2 mode requires SAML right now
        if new_config.a2_mode.unwrap_or(false) {
            new_config.saml = Some(true)
        }

        fips::merge_fips_options_and_config(
            self.fips,
            self.fips_git_port,
            self.fips_custom_cert_filename,
            new_config,
        )
    }
}

fn with_default<'a>(val: &'a str, default: &'a str, local: &bool) -> &'a str {
    if !local || !val.is_empty() {
        val
    } else {
        default
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Run one or more phase jobs")
        .args(&vec![patchset_arg(), project_arg(), local_arg()])
        .args(&make_arg_vec![
            "-j --job-root=[root] 'Path to the job root'",
            "-g --git-url=[url] 'Git URL (-u -s -e -o ignored if used)'",
            "-C --change=[change] 'Feature branch name'",
            "-b --branch=[branch] 'Branch to merge'",
            "-S --shasum=[gitsha] 'Git SHA of change'",
            "--change-id=[id] 'The change ID'",
            "--skip-default 'skip default'",
            "--docker=[image] 'Docker image'"
        ])
        .args_from_usage(
            "<stage> 'Stage for the run'
                          <phases> 'One or more phases'",
        )
        .args(&u_e_s_o_args())
        .args(&pipeline_arg())
        .args(&project_specific_args())
        .args(&vec![a2_mode_arg()])
}
