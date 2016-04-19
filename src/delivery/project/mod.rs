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

use hyper::client::response::Response;
use hyper::status::StatusCode;
use utils::say::{say, sayln};
use utils::{self, walk_tree_for_path};
use errors::{DeliveryError, Kind};
use std::path::{Path, PathBuf};
use http::APIClient;
use git;
use config::Config;

fn call_api<F>(closure: F) where F : Fn() -> Result<Response, Kind> {
    match closure() {
        Ok(_) => {
            sayln("white", "done");
        },
        Err(e) => {
            // Why we dont pass the Err() to fail?
            // We should fail in this point since we could
            // endup in an unknown state because we assume
            // that everything is Ok()
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
}

pub fn import(config: &Config, path: &PathBuf) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());

    // Init && config local repo if necessary
    try!(git::init_repo(path));
    let url = try!(config.delivery_git_ssh_url());
    if try!(git::config_repo(&url, path)) {
        sayln("white", "Remote 'delivery' added to git config!");
    } else {
        sayln("red", "Remote named 'delivery' already exists - not modifying");
        // We should verify that the remote is the correct one
        // if not, delete it and create the right one.
        // Or fail saying that it doesn't match
    }

    let client = try!(APIClient::from_config(config));

    if ! client.project_exists(&org, &proj) {
        say("white", "Creating project: ");
        say("magenta", &format!("{} ", proj));
        call_api(|| client.create_project(&org, &proj));
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
    call_api(|| client.create_pipeline(&org, &proj, "master"));
    return Ok(())
}

/// Search for the project root directory
///
/// We will walk through the provided path tree until we find the
/// git config (`.git/config`) annd then we will extract the root
/// directory.
///
/// # Examples
///
/// Having this directory tree:
/// /delivery-cli
///  ├── .git
///  │   └── config
///  ├── src
///  │   └── delivery
///  └── features
///
/// ```
/// use std::env;
/// use delivery::project::root_dir;
///
/// let root = env::current_dir().unwrap();
///
/// // Stepping into `delivery-cli/src/delivery`
/// let mut delivery_src = env::current_dir().unwrap();
/// delivery_src.push("src/delivery");
///
/// assert_eq!(root, root_dir(&delivery_src.as_path()).unwrap());
/// ```
pub fn root_dir(dir: &Path) -> Result<PathBuf, DeliveryError> {
    match walk_tree_for_path(&PathBuf::from(&dir), ".git/config") {
        Some(p) => {
           let git_d = p.parent().unwrap();
           let root_d = git_d.parent().unwrap();
           debug!("found project root dir: {:?}", root_d);
           Ok(PathBuf::from(root_d))
        },
        None => Err(DeliveryError{kind: Kind::NoGitConfig,
                                  detail: Some(format!("current directory: {:?}",
                                                       dir))})
    }
}

// Return the project name from the current path
pub fn project_from_cwd() -> Result<String, DeliveryError> {
    let cwd = try!(self::root_dir(&utils::cwd()));
    Ok(cwd.file_name().unwrap().to_str().unwrap().to_string())
}

// Return the project name or try to extract it from the current path
pub fn project_or_from_cwd(proj: &str) -> Result<String, DeliveryError> {
    if proj.is_empty() {
        project_from_cwd()
    } else {
        Ok(proj.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::root_dir;

    #[test]
    fn detect_error_if_root_project_is_not_a_git_repo() {
        // This path doesn't even exist
        // So we will expect to throw an Err(_)
        let lib_path = Path::new("/project/src/libraries");
        match root_dir(&lib_path) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true)
        }
    }
}

