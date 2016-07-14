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
use cli::{for_arg, patchset_arg, value_of};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "diff";

#[derive(Debug)]
pub struct DiffClapOptions<'n> {
    pub change: &'n str,
    pub patchset: &'n str,
    pub pipeline: &'n str,
    pub local: bool,
}
impl<'n> Default for DiffClapOptions<'n> {
    fn default() -> Self {
        DiffClapOptions {
            change: "",
            patchset: "",
            pipeline: "master",
            local: false,
        }
    }
}

impl<'n> DiffClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        DiffClapOptions {
            change: value_of(&matches, "change"),
            patchset: value_of(&matches, "patchset"),
            pipeline: value_of(&matches, "for"),
            local: matches.is_present("local"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Display diff for a change")
        .args(&vec![for_arg(), patchset_arg()])
        .args_from_usage(
            "<change> 'Name of the feature branch to compare'
            -l --local \
            'Diff against the local branch HEAD'")
}
