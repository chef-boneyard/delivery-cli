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
use cli::Options;
use cli::arguments::{api_port_arg, config_path_arg, u_e_s_o_args, value_of};
use config::Config;
use types::DeliveryResult;

pub const SUBCOMMAND_NAME: &'static str = "api";

#[derive(Debug)]
pub struct ApiClapOptions<'n> {
    pub method: &'n str,
    pub path: &'n str,
    pub data: &'n str,
    pub server: &'n str,
    pub api_port: &'n str,
    pub ent: &'n str,
    pub user: &'n str,
}

impl<'n> Default for ApiClapOptions<'n> {
    fn default() -> Self {
        ApiClapOptions {
            method: "",
            path: "",
            data: "",
            server: "",
            api_port: "",
            ent: "",
            user: "",
        }
    }
}
impl<'n> ApiClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        ApiClapOptions {
            method: value_of(&matches, "method"),
            path: value_of(&matches, "path"),
            data: value_of(&matches, "data"),
            server: value_of(&matches, "server"),
            api_port: value_of(&matches, "api-port"),
            ent: value_of(&matches, "ent"),
            user: value_of(&matches, "user"),
        }
    }
}

impl<'n> Options for ApiClapOptions<'n> {
    fn merge_options_and_config(&self, config: Config) -> DeliveryResult<Config> {
        let new_config = config
            .set_user(&self.user)
            .set_server(&self.server)
            .set_api_port(&self.api_port)
            .set_enterprise(&self.ent);
        Ok(new_config)
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Helper to call Delivery's HTTP API")
        .arg(api_port_arg())
        .args(&vec![config_path_arg()])
        .arg(
            Arg::from_usage("<method> 'HTTP method for the request'")
                .takes_value(false)
                .possible_values(&["get", "put", "post", "delete"]),
        )
        .args_from_usage("<path> 'Path for request URL'")
        .arg(
            Arg::with_name("data")
                .long("data")
                .short("d")
                .help("Data to send for PUT/POST request")
                .takes_value(true)
                .multiple(false)
                .number_of_values(1)
                .use_delimiter(false),
        )
        .args(&u_e_s_o_args())
}
