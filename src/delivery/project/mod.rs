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
    /// Create a new `SourceCodeProvider`. Returns an error result if
    /// required configuration values are missing. Expects to find
    /// `scp`, `repository`, `scp-organization`, `branch`, and `ssl`.
    pub fn new(scp: &str, repo: &str, org: &str, branch: &str,
               no_ssl: bool) -> Result<SourceCodeProvider, DeliveryError> {
        let scp_kind = match scp {
            "github" => Type::Github,
            "bitbucket" => Type::Bitbucket,
            _ => return Err(DeliveryError{ kind: Kind::UnknownProjectType, detail:None })
        };
        if repo.to_string().is_empty()
            || org.to_string().is_empty()
            || branch.to_string().is_empty() {
            match scp_kind {
                Type::Github => return Err(
                    DeliveryError{
                        kind: Kind::OptionConstraint,
                        detail: Some(format!("Missing Github Source Code Provider attributes, specify: \
                                              repo-name, org-name and pipeline(default: master)"))
                    }
                ),
                Type::Bitbucket => return Err(
                    DeliveryError{
                        kind: Kind::OptionConstraint,
                        detail: Some(format!("Missing Bitbucket Source Code Provider attributes, specify: \
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
            verify_ssl: !no_ssl,
        })
    }

    /// Verify if the SCP is configured on the Delivery Server
    pub fn verify_server_config(&self, client: &APIClient) -> Result<(), DeliveryError> {
        match self.kind {
            Type::Github => {
                let scp_config = try!(client.get_github_server_config());
                if scp_config.is_empty() {
                    return Err(DeliveryError{ kind: Kind::NoGithubSCPConfig, detail: None })
                }
            },
            Type::Bitbucket => {
                let scp_config = try!(client.get_bitbucket_server_config());
                if scp_config.is_empty() {
                    return Err(DeliveryError{ kind: Kind::NoBitbucketSCPConfig, detail: None })
                }
            }
        }
        Ok(())
    }
}

/// Create a Delivery Project
///
/// This method will create a Delivery Project depending on the SCP that we specify,
/// either a Github, Bitbucket or Delivery (default). It also creates a pipeline,
/// adds the `delivery` remote and push the content of the local repo to the Server.
pub fn create_on_server(config: &Config,
              scp: Option<SourceCodeProvider>, local: &bool) -> Result<(), DeliveryError> {
    if *local {
        return Ok(())
    }
    let org = try!(config.organization());
    let proj = try!(config.project());
    let path = try!(root_dir(&utils::cwd()));
    try!(git::init_repo(&path));
    let client = try!(APIClient::from_config(config));

    if client.project_exists(&org, &proj) {
        say("white", "Project ");
        say("magenta", &format!("{} ", proj));
        sayln("white", "already exists.");
    } else {
        match scp {
            Some(scp_config) => try!(create_scp_project(client, config, &path, scp_config)),
            None => try!(create_delivery_project(client, config, &path))
        }
    }
    Ok(())
}

/// Create a Delivery Project with Delivery as SCP (default)
fn create_delivery_project(client: APIClient, config: &Config,
                           path: &PathBuf) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());
    let pipe = try!(config.pipeline());
    say("white", "Creating ");
    say("magenta", "delivery");
    say("white", " project: ");
    say("magenta", &format!("{} ", proj));
    try!(client.create_delivery_project(&org, &proj));
    try!(push_project_content_to_delivery(config, path));
    say("white", "Creating ");
    say("magenta", &format!("{} ", pipe));
    say("white", " pipeline for project: ");
    say("magenta", &format!("{}: ", proj));
    try!(client.create_pipeline(&org, &proj, &pipe));
    Ok(())
}

/// Add the `delivery` remote to the local git reposiory and
/// then push local content to the Delivery Server
fn push_project_content_to_delivery(config: &Config, path: &PathBuf) -> Result<(), DeliveryError> {
    let url = try!(config.delivery_git_ssh_url());
    if try!(git::config_repo(&url, path)) {
        sayln("white", "Remote 'delivery' added to git config!");
    } else {
        sayln("red", "Remote named 'delivery' already exists - not modifying");
        // We should verify that the remote is the correct one
        // if not, delete it and create the right one.
        // Or fail saying that it doesn't match
    }
    say("white", "Checking for content on the git remote ");
    say("magenta", "delivery: ");
    if git::server_content() {
        sayln("red", "Found commits upstream, not pushing local commits");
        Ok(())
    } else {
        sayln("white", "Pushing local content to server:");
        git::git_push_master()
    }
}

/// Create a Delivery Project with Bitbucket or Github as SCP
fn create_scp_project(client: APIClient, config: &Config,
                      path: &PathBuf, scp: SourceCodeProvider) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());
    try!(scp.verify_server_config(&client));
    say("white", "Creating ");
    match scp.kind {
        Type::Bitbucket => {
            say("magenta", "bitbucket");
            say("white", " project: ");
            say("magenta", &format!("{} ", proj));
            try!(client.create_bitbucket_project(&org, &proj, &scp.repo_name,
                                                 &scp.organization, &scp.branch));
            try!(push_project_content_to_delivery(config, path));
        },
        Type::Github => {
            say("magenta", "github");
            say("white", " project: ");
            say("magenta", &format!("{} ", proj));
            try!(client.create_github_project(&org, &proj, &scp.repo_name,
                                              &scp.organization, &scp.branch, scp.verify_ssl));
        }
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

/// Return the project name from the current path
pub fn project_from_cwd() -> Result<String, DeliveryError> {
    let cwd = try!(self::root_dir(&utils::cwd()));
    Ok(cwd.file_name().unwrap().to_str().unwrap().to_string())
}

/// Return the project name or try to extract it from the current path
pub fn project_or_from_cwd(proj: &str) -> Result<String, DeliveryError> {
    if proj.is_empty() {
        project_from_cwd()
    } else {
        Ok(proj.to_string())
    }
}

/// Initialize a Delivery project
///
/// This method will init a Delivery project doing the following:
/// * Create the project in Delivery. (It knows how to link the project to a
///   Github or Bitbucket SCP)
/// * Add the `delivery` remote (Only Delivery & Bitbucket projects)
/// * Push local content to Delivery (Only Delivery & Bitbucket projects)
/// * Create a Pipeline
/// * Create a feature branch called `add-delivery-config` to:
///     * Create a build-cookbook
///     * Create the `.delivery/config.json`
/// * Finally submit a cli::review (Only for Delivery & Bitbucket projects)
///
pub fn init(config: Config, no_open: &bool, skip_build_cookbook: &bool,
            local: &bool, scp: Option<SourceCodeProvider>) -> Result<(), DeliveryError> {
    let project_path = try!(root_dir(&utils::cwd()));
    let config_json = config.config_json.clone();
    try!(create_on_server(&config, scp.clone(), local));
    try!(create_feature_branch(&project_path));
    try!(generate_build_cookbook(skip_build_cookbook));
    try!(generate_delivery_config(config_json));
    try!(trigger_review(config, scp, &no_open, &local));
    Ok(())
}

fn generate_delivery_config(config_json: Option<String>) -> Result<(), DeliveryError> {
    let project_path = try!(root_dir(&utils::cwd()));
    if let Some(json) = config_json {
        let json_path = PathBuf::from(json);
        DeliveryConfig::copy_config_file(&json_path, &project_path)
    } else {
        DeliveryConfig::init(&project_path)
    }
}

fn trigger_review(config: Config, scp: Option<SourceCodeProvider>,
                 no_open: &bool, local: &bool) -> Result<(), DeliveryError> {
    if *local {
        return Ok(())
    }
    let pipeline = try!(config.pipeline());
    match scp {
        Some(s) => {
            match s.kind {
                Type::Bitbucket => {
                    try!(cli::review(&pipeline, &false, no_open, &false));
                },
                Type::Github => {
                    // For now, delivery review doesn't works for Github projects
                    // TODO: Make it work in github
                    sayln("green", "Push add-delivery-config branch and create Pull Request");
                }
            }
        },
        None => try!(cli::review(&pipeline, &false, no_open, &false))
    }
    Ok(())
}

fn create_feature_branch(project_path: &PathBuf) -> Result<(), DeliveryError> {
    // In the case of an Err() we could roll back although
    // it could be a good idea to verify and if not, create it.
    say("white", "Create and checkout add-delivery-config feature branch: ");
    try!(git::git_command(&["checkout", "-b", "add-delivery-config"], project_path));
    sayln("green", "done");
    Ok(())
}

fn generate_build_cookbook(skip_build_cookbook: &bool) -> Result<(), DeliveryError> {
    if *skip_build_cookbook {
        return Ok(())
    }
    sayln("white", "Generating build cookbook skeleton");
    let project_path = try!(root_dir(&utils::cwd()));
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
        .arg(pcb_dir)
        .current_dir(&project_path);

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
    try!(git::git_command(&["add", ".delivery/build-cookbook"], &project_path));
    try!(git::git_command(&["commit", "-m", "Add Delivery build cookbook"], &project_path));
    sayln("green", "done");
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

