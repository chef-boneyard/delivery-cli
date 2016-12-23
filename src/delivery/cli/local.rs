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
use clap::{App, SubCommand, ArgMatches, Arg};
use delivery_config::project::Phase;
use cli::arguments::value_of;

pub const SUBCOMMAND_NAME: &'static str = "local";

#[derive(Debug)]
pub struct LocalClapOptions<'n> {
    pub phase: Option<Phase>,
    pub remote_toml: &'n str
}

impl<'n> Default for LocalClapOptions<'n> {
    fn default() -> Self {
        LocalClapOptions {
            phase: None,
            remote_toml: ""
        }
    }
}

impl<'n> LocalClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        let phase = match value_of(matches, "phase") {
            "unit" => Some(Phase::Unit),
            "lint" => Some(Phase::Lint),
            "syntax" => Some(Phase::Syntax),
            "provision" => Some(Phase::Provision),
            "deploy" => Some(Phase::Deploy),
            "smoke" => Some(Phase::Smoke),
            "functional" => Some(Phase::Functional),
            "cleanup" => Some(Phase::Cleanup),
            _ => None
        };

        LocalClapOptions {
            phase: phase,
            remote_toml: value_of(&matches, "remote-project-toml")
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Run Delivery phases on your local workstation.")
        .arg(Arg::from_usage("<phase> 'Delivery phase to execute'")
             .takes_value(false)
             .possible_values(&["unit", "lint", "syntax", "provision",
                                "deploy", "smoke", "functional", "cleanup"]))
        .args_from_usage("-r --remote-project-toml=[remote-url] 'URL for remote project.toml'")
}
