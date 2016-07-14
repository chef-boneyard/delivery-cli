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
use cli::{for_arg, no_open_arg, value_of, auto_bump};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "review";

#[derive(Debug)]
pub struct ReviewClapOptions<'n> {
    pub pipeline: &'n str,
    pub no_open: bool,
    pub auto_bump: bool,
    pub edit: bool,
}
impl<'n> Default for ReviewClapOptions<'n> {
    fn default() -> Self {
        ReviewClapOptions {
            pipeline: "master",
            no_open: false,
            auto_bump: false,
            edit: false,
        }
    }
}

impl<'n> ReviewClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        ReviewClapOptions {
            pipeline: value_of(&matches, "for"),
            no_open: matches.is_present("no-open"),
            auto_bump: matches.is_present("auto-bump"),
            edit: matches.is_present("edit"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Submit current branch for review")
        .args(&vec![for_arg(), no_open_arg(), auto_bump()])
        .args_from_usage("-e --edit 'Edit change title and description'")
}
