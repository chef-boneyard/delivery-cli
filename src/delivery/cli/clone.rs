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
use cli::{u_e_s_o_args, value_of};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "clone";

#[derive(Debug)]
pub struct CloneClapOptions<'n> {
    pub project: &'n str,
    pub user: &'n str,
    pub server: &'n str,
    pub ent: &'n str,
    pub org: &'n str,
    pub git_url: &'n str,
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
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Clone a project repository")
        .args_from_usage(
            "<project> 'Name of project to clone'
            -g --git-url=[url] \
            'Git URL (-u -s -e -o ignored if used)'")
        .args(&u_e_s_o_args())
}
