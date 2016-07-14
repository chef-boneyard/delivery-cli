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
use cli::{for_arg, config_path_arg, u_e_s_o_args, value_of};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "setup";

#[derive(Debug)]
pub struct SetupClapOptions<'n> {
    pub user: &'n str,
    pub server: &'n str,
    pub ent: &'n str,
    pub org: &'n str,
    pub path: &'n str,
    pub pipeline: &'n str,
}

impl<'n> Default for SetupClapOptions<'n> {
    fn default() -> Self {
        SetupClapOptions {
            user: "",
            server: "",
            ent: "",
            org: "",
            path: "",
            pipeline: "master",
        }
    }
}

impl<'n> SetupClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        SetupClapOptions {
            user: value_of(&matches, "user"),
            server: value_of(&matches, "server"),
            ent: value_of(&matches, "ent"),
            org: value_of(&matches, "org"),
            path: value_of(&matches, "config-path"),
            pipeline: value_of(&matches, "for"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Write a config file capturing specified options")
        .args(&vec![for_arg(), config_path_arg()])
        .args(&u_e_s_o_args())
}
