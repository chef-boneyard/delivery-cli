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
use utils::{self, walk_tree_for_path, mkdir_recursive};
use utils::path_ext::is_dir;
use errors::{DeliveryError, Kind};
use delivery_config::DeliveryConfig;
use std::error;
use std::path::{Path, PathBuf};
use http::APIClient;
use git;
use cli;
use config::Config;

#[derive(Debug, Clone)]
pub enum Type {
    Bitbucket,
    Github
}

#[derive(Debug, Clone)]
pub struct SourceCodeProvider {
    pub repo_name: String,
    pub organization: String,
    pub branch: String,
    pub verify_ssl: bool,
    pub kind: Type,
}

impl SourceCodeProvider {
    pub fn new(scp: &str, repo: &str, org: &str, branch: &str,
               ssl: bool) -> Result<SourceCodeProvider, DeliveryError> {
        let scp_kind = match scp {
            "github" => Type::Github,
            "bitbucket" => Type::Bitbucket,
            _ => return Err(DeliveryError{ kind: Kind::OptionConstraint, detail:None })
        };
        // Verify all SCP Attributes are valid
        if repo.to_string().is_empty()
            || org.to_string().is_empty()
            || branch.to_string().is_empty() {
            match scp_kind {
                Type::Github => return Err(
                    DeliveryError{
                        kind: Kind::OptionConstraint,
                        detail: Some(format!("To initialize a Github project you have to specify: \
                                              repo-name, org-name and pipeline(default: master)"))
                    }
                ),
                Type::Bitbucket => return Err(
                    DeliveryError{
                        kind: Kind::OptionConstraint,
                        detail: Some(format!("To initialize a Bitbucket project you have to specify: \
                                              repo-name, project-key and pipeline(default: master)"))
                    }
                ),
            }
        }
        Ok(SourceCodeProvider {
            kind: scp_kind,
            repo_name: repo.to_string(),
            organization: org.to_string(),
            branch: branch.to_string(),
            verify_ssl: ssl,
        })
    }
}

fn call_api<F>(closure: F) where F : Fn() -> Result<Response, Kind> {
    match closure() {
        Ok(_) => {
            sayln("green", "done");
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

pub fn create(config: &Config, path: &PathBuf,
              scp: Option<SourceCodeProvider>) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());

    // Init && config local repo if necessary
    try!(git::init_repo(path));

    let client = try!(APIClient::from_config(config));

    if client.project_exists(&org, &proj) {
        say("white", "Project ");
        say("magenta", &format!("{} ", proj));
        sayln("white", "already exists.");
    } else {
        match scp {
            Some(s) => try!(create_scp_project(client, &org, &proj, s)),
            None => {
                try!(create_delivery_project(client, &org, &proj, config, path))
            }
        }
    }
    Ok(())
}

fn create_delivery_project(client: APIClient, org: &str, proj: &str,
                           config: &Config, path: &PathBuf,) -> Result<(), DeliveryError> {
    let pipe = try!(config.pipeline());
    let url = try!(config.delivery_git_ssh_url());
    if try!(git::config_repo(&url, path)) {
        sayln("white", "Remote 'delivery' added to git config!");
    } else {
        sayln("red", "Remote named 'delivery' already exists - not modifying");
        // We should verify that the remote is the correct one
        // if not, delete it and create the right one.
        // Or fail saying that it doesn't match
    }
    say("white", "Creating ");
    say("magenta", "delivery");
    say("white", " project: ");
    say("magenta", &format!("{} ", proj));
    call_api(|| client.create_delivery_project(&org, &proj));
    say("white", "Checking for content on the git remote ");
    say("magenta", "delivery: ");
    if git::server_content() {
        sayln("red", "Found commits upstream, not pushing local commits.");
    } else {
        sayln("white", "No upstream content; pushing local content to server.");
        // Why here we push to master and not the pipeline ¿?
        //let _ = git::git_push_pipeline(&pipe);
        let _ = git::git_push_master();
    }
    say("white", "Creating ");
    say("magenta", &format!("{} ", pipe));
    say("white", " pipeline for project: ");
    say("magenta", &format!("{} ", proj));
    say("white", "... ");
    call_api(|| client.create_pipeline(&org, &proj, &pipe));
    Ok(())
}

fn create_scp_project(client: APIClient, org: &str, proj: &str,
                      scp: SourceCodeProvider) -> Result<(), DeliveryError> {
    say("white", "Creating ");
    match scp.kind {
        Type::Bitbucket => {
            say("magenta", "bitbucket");
            say("white", " project: ");
            say("magenta", &format!("{} ", proj));
            call_api(|| client.create_bitbucket_project(&org, &proj, &scp.repo_name,
                                                        &scp.organization, &scp.branch));
        },
        Type::Github => {
            say("magenta", "github");
            say("white", " project: ");
            say("magenta", &format!("{} ", proj));
            call_api(|| client.create_github_project(&org, &proj, &scp.repo_name,
                                                     &scp.organization, &scp.branch,
                                                     scp.verify_ssl));
        },
        //_ => Err(),
    }
    Ok(())
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

pub fn init(config: Config, no_open: &bool, skip_build_cookbook: &bool,
        local: &bool, scp: Option<SourceCodeProvider>) -> Result<(), DeliveryError> {
    let project_path = try!(root_dir(&utils::cwd()));

    if !local {
        try!(create(&config, &project_path, scp.clone()));
    }

    // From here we must create a new feature branch
    // since we are going to start modifying the repo.
    // in the case of an Err() we could roll back by
    // reseting to the pipeline/branch (git reset --hard)
    // Although it could be a good idea to verify and if not, create it.
    say("white", "Create and checkout add-delivery-config feature branch: ");
    try!(git::git_command(&["checkout", "-b", "add-delivery-config"], &project_path));
    sayln("green", "done");

    // we want to generate the build cookbook by default. let the user
    // decide to skip if they don't want one.
    if ! *skip_build_cookbook {

        sayln("white", "Generating build cookbook skeleton");

        let pcb_dir = match utils::home_dir(&[".delivery/cache/generator-cookbooks/pcb"]) {
            Ok(p) => p,
            Err(e) => return Err(e)
        };

        if is_dir(&pcb_dir) {
            sayln("yellow", "Cached copy of build cookbook generator exists; skipping git clone.");
        } else {
            sayln("white", &format!("Cloning build cookbook generator dir {:#?}", pcb_dir));
            // Lets not force the user to use this git repo.
            // Adding an option --pcb PATH
            // Where PATH:
            //    * Local path
            //    * Git repo
            //    * Supermarket?
            try!(git::clone(&pcb_dir.to_string_lossy(),
                            "https://github.com/chef-cookbooks/pcb"));
        }

        // Generate the cookbook
        let dot_delivery = Path::new(".delivery");
        try!(mkdir_recursive(dot_delivery));
        let mut gen = utils::make_command("chef");
        gen.arg("generate")
            .arg("cookbook")
            .arg(".delivery/build-cookbook")
            .arg("-g")
            .arg(pcb_dir);

        match gen.output() {
            Ok(o) => o,
            Err(e) => return Err(
                        DeliveryError {
                            kind: Kind::FailedToExecute,
                            detail: Some(format!(
                                        "failed to execute chef generate: {}",
                                        error::Error::description(&e)
                                    ))
                        })
        };

        sayln("green", &format!("PCB generate: {:#?}", gen));
        say("white", "Git add and commit of build-cookbook: ");
        try!(git::git_command(&["add", ".delivery/build-cookbook"], &utils::cwd()));
        try!(git::git_command(&["commit", "-m", "Add Delivery build cookbook"], &utils::cwd()));
        sayln("green", "done");
    }

    // now to adding the .delivery/config.json, this uses our
    // generated build cookbook always, so we no longer need a project
    // type.
    try!(DeliveryConfig::init(&utils::cwd()));

    if !local {
        // For now, delivery review only works for projects that the SCP is delivery
        // TODO: Make it work in bitbucket and github
        match scp {
            Some(_) => sayln("green", "Push add-delivery-config branch and create Pull Request"),
            None => {
                let pipeline = try!(config.pipeline());
                try!(cli::review(&pipeline, &false, no_open, &false));
            }
        }
    }
    Ok(())
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

