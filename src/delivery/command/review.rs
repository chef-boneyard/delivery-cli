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

use std;
use fips;
use cli::review::ReviewClapOptions;
use config::Config;
use utils;
use utils::say::{sayln, say};
use errors::DeliveryError;
use types::{DeliveryResult, ExitCode};
use cookbook;
use git::{self, ReviewResult};
use http;
use delivery_config::DeliveryConfig;
use command::Command;
use project;

pub struct ReviewCommand<'n> {
    pub options: &'n ReviewClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for ReviewCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        try!(super::verify_and_repair_git_remote(&self.config));
        try!(fips::setup_and_start_stunnel_if_fips_mode(&self.config, child_processes));
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");

        let config_ref = self.config;
        let target = validate!(config_ref, pipeline);
        if let Some(should_bump) = self.config.auto_bump {
            if should_bump {
                let project =  validate!(config_ref, project);
                let project_root = try!(project::root_dir(&utils::cwd()));
                try!(DeliveryConfig::validate_config_file(&project_root));
                try!(cookbook::bump_version(&project_root, &target, &project))
            }
        }

        let head = try!(git::get_head());
        say("white", "Review for change ");
        say("yellow", &head);
        say("white", " targeted for pipeline ");
        sayln("magenta", &target);
        let review = try!(project::review(&target, &head));

        if self.options.edit {
            try!(edit_change(&self.config, &review));
        }

        for line in review.messages.iter() {
            sayln("white", line);
        }

        match project::handle_review_result(&review, &self.options.no_open) {
            Ok(result) => {
                match result {
                    Some(url) => {sayln("magenta", &url)},
                    None => {}
                }
            },
            Err(_) => {
                sayln("yellow", "We could not open the review in the browser for you.");
                sayln("yellow", "Make sure there is a program that can open HTML files in your path \
                                 or pass --no-open to bypass attempting to open this review in a browser.");
            }
        }
        Ok(0)
    }
}

fn edit_change(config: &Config,
               review: &ReviewResult) -> Result<(), DeliveryError> {
    let proj = try!(config.project());
    match review.change_id {
        Some(ref change_id) => {
            let change0 = try!(http::change::get(&config, &change_id));
            let text0 = format!("{}\n\n{}\n",
                                change0.title, change0.description);
            let text1 = try!(utils::open::edit_str(&proj, &text0));
            let change1 = try!(http::change::Description::parse_text(&text1));
            Ok(try!(http::change::set(&config, &change_id, &change1)))
        },
        None => Ok(())
    }
}
