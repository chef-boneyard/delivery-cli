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
use cli::arguments::value_of;
use delivery_config::project::{Phase, Stage};

pub const SUBCOMMAND_NAME: &'static str = "local";

#[derive(Debug)]
pub struct LocalClapOptions<'n> {
    pub phase: Option<Phase>,
    pub stage: Option<Stage>,
    pub remote_toml: Option<&'n str>,
}

impl<'n> Default for LocalClapOptions<'n> {
    fn default() -> Self {
        LocalClapOptions {
            phase: None,
            stage: None,
            remote_toml: None,
        }
    }
}

impl<'n> LocalClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        let mut phase: Option<Phase> = None;
        let mut stage: Option<Stage> = None;
        match value_of(matches, "stage_phase") {
            "unit" => phase = Some(Phase::Unit),
            "lint" => phase = Some(Phase::Lint),
            "syntax" => phase = Some(Phase::Syntax),
            "provision" => phase = Some(Phase::Provision),
            "deploy" => phase = Some(Phase::Deploy),
            "smoke" => phase = Some(Phase::Smoke),
            "functional" => phase = Some(Phase::Functional),
            "cleanup" => phase = Some(Phase::Cleanup),
            "verify" => stage = Some(Stage::Verify),
            "acceptance" => stage = Some(Stage::Acceptance),
            "all" => stage = Some(Stage::All),
            _ => {}
        };

        let url = match value_of(&matches, "remote-project-toml") {
            "" => None,
            u => Some(u),
        };

        LocalClapOptions {
            phase: phase,
            stage: stage,
            remote_toml: url,
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Run Delivery phases on your local workstation.")
        .arg(
            Arg::with_name("stage_phase")
                .takes_value(false)
                .required(true)
                .possible_values(&[
                    "unit",
                    "lint",
                    "syntax",
                    "provision",
                    "deploy",
                    "smoke",
                    "functional",
                    "cleanup",
                    // Stages
                    "verify",
                    "acceptance",
                    "all",
                ])
                .help(
                    "Automate phase or stage to execute locally.\n\nAvailable phases: [unit, \
                     lint, syntax, provision, deploy, smoke, functional, cleanup]\n\nStages \
                     will execute a series of phases in the following order:\nverify: [unit, \
                     lint, syntax]\nacceptance: [provision, deploy, smoke, functional, \
                     cleanup]\nall: [unit, lint, syntax, provision, deploy, smoke, functional, \
                     cleanup]\n\n",
                ),
        )
        .args_from_usage("-r --remote-project-toml=[remote-url] 'URL for remote project.toml'")
}
