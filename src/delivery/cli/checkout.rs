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

pub const SUBCOMMAND_NAME: &'static str = "checkout";

#[derive(Debug)]
pub struct CheckoutClapOptions<'n> {
    pub pipeline: &'n str,
    pub change: &'n str,
    pub patchset: &'n str,
}
impl<'n> Default for CheckoutClapOptions<'n> {
    fn default() -> Self {
        CheckoutClapOptions {
            pipeline: "master",
            change: "",
            patchset: "",
        }
    }
}

impl<'n> CheckoutClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        CheckoutClapOptions {
            pipeline: value_of(&matches, "for"),
            change: value_of(&matches, "change"),
            patchset: value_of(&matches, "patchset"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Create a local branch tracking an in-progress change")
        .args(&vec![for_arg(), patchset_arg()])
        .args_from_usage("<change> 'Name of the feature branch to checkout'")
}
