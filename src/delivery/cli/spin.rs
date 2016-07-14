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
use cli::value_of;
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "spin";

#[derive(Debug)]
pub struct SpinClapOptions {
    pub time: u64,
}
impl Default for SpinClapOptions {
    fn default() -> Self {
        SpinClapOptions {
            time: 5,
        }
    }
}

impl SpinClapOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let t = value_of(&matches, "time");
        if t.is_empty() {
            Default::default()
        } else {
            SpinClapOptions {
                time: t.parse::<u64>().unwrap(),
            }
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("test the spinner")
        .args_from_usage("-t --time=[TIME] 'How many seconds to spin. default:5'")
}
