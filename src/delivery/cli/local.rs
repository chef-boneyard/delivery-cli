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
use cli::value_of;

pub const SUBCOMMAND_NAME: &'static str = "local";

#[derive(Debug)]
pub struct LocalClapOptions {
    pub phase: Option<Phase>
}

impl Default for LocalClapOptions {
    fn default() -> Self {
        LocalClapOptions { phase: None }
    }
}

impl LocalClapOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let phase = match value_of(matches, "phase") {
            "unit" => Some(Phase::Unit),
            "lint" => Some(Phase::Lint),
            "syntax" => Some(Phase::Syntax),
            "provision" => Some(Phase::Provision),
            "deploy" => Some(Phase::Deploy),
            "smoke" => Some(Phase::Smoke),
            "cleanup" => Some(Phase::Cleanup),
            _ => None
        };

        LocalClapOptions { phase: phase }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Run Delivery phases on your local workstation.")
        .arg(Arg::from_usage("<phase> 'Delivery phase to execute'")
             .takes_value(false)
             .possible_values(&["unit", "lint", "syntax", "provision",
                                "deploy", "smoke", "cleanup"]))
}
