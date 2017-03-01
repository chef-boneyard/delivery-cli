//
// Copyright:: Copyright (c) 2017 Chef Software, Inc.
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

use cli::arguments::{api_port_arg, server_arg, value_of};
use clap::{App, SubCommand, ArgMatches};
use cli::Options;
use types::DeliveryResult;
use config::Config;

pub const SUBCOMMAND_NAME: &'static str = "status";

#[derive(Debug)]
pub struct StatusClapOptions<'n> {
    pub api_port: &'n str,
    pub json: bool,
    pub server: &'n str,
}

impl<'n> Default for StatusClapOptions<'n> {
    fn default() -> Self {
        StatusClapOptions {
            api_port: "",
            json: false,
            server: "",
        }
    }
}

impl<'n> StatusClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        StatusClapOptions {
            api_port: value_of(&matches, "api-port"),
            json: matches.is_present("json"),
            server: value_of(&matches, "server"),
        }
    }
}

impl<'n> Options for StatusClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let new_config = config.set_api_port(&self.api_port)
            .set_server(&self.server);
        Ok(new_config)
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Get status information about the Automate Server's _status endpoint")
        .arg(api_port_arg())
        .args_from_usage("--json 'Output the raw JSON from the _status endpoint'")
        .arg(server_arg())
}
