//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
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

use hyper::status::StatusCode;
use utils::say::{say, sayln};
use errors::{DeliveryError, Kind};
use std::path::{PathBuf};
use http::APIClient;
use git;
use config::Config;

pub fn import(config: &Config, path: &PathBuf) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());

    // Init && config local repo if necessary
    try!(git::init_repo(path));
    let url = try!(config.delivery_git_ssh_url());
    if try!(git::config_repo(&url, path)) {
        sayln("white", "Remote 'delivery' added to git config!");
    }

    let client = try!(APIClient::from_config(config));

    if ! client.project_exists(&org, &proj) {
        say("white", "Creating project: ");
        sayln("magenta", &format!("{} ", proj));
        let _ = client.create_project(&org, &proj);
    } else {
        say("white", "Project ");
        say("magenta", &format!("{} ", proj));
        sayln("white", "already exists.");
    }

    say("white", "Checking for content on the git remote ");
    say("magenta", "delivery: ");
    if git::server_content() {
        sayln("red", "Found commits upstream, not pushing local commits.");
    } else {
        sayln("white", "No upstream content; pushing local content to server.");
        let _ = git::git_push_master();
    }

    say("white", "Creating master pipeline for project: ");
    say("magenta", &format!("{} ", proj));
    say("white", "... ");
    match client.create_pipeline(&org, &proj, "master") {
         Ok(_) => {
            sayln("white", "done");
        },
        Err(e) => {
            match e {
                Kind::ApiError(StatusCode::Conflict, _) => {
                    sayln("white", " already exists.");
                },
                Kind::ApiError(code, Ok(msg)) => {
                    sayln("red", &format!("{} {}", code, msg));
                },
                _ => {
                    sayln("red", &format!("Other error: {:?}", e));
                }
            }
        }
    }
    return Ok(())
}

