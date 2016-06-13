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
use std::path::{Path, PathBuf};
use http::APIClient;
use git;
use cli;
use config::Config;
use std::process::Output;

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
    let path = try!(root_dir(&utils::cwd()));
    let client = try!(APIClient::from_config(config));

    match scp {
        Some(scp_config) => try!(create_scp_project(client, config, &path, scp_config)),
        None => {
            try!(create_delivery_project(&client, config));
            try!(push_project_content_to_delivery(config, &path));
            try!(create_delivery_pipeline(&client, config));
        }
    }
    Ok(())
}

/// Create a Delivery Pipeline
fn create_delivery_pipeline(client: &APIClient, config: &Config) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());
    let pipe = try!(config.pipeline());
    if client.pipeline_exists(&org, &proj, &pipe) {
        say("white", "Pipeline ");
        say("magenta", &format!("{} ", pipe));
        sayln("white", "already exists.");
    } else {
        say("white", "Creating ");
        say("magenta", &format!("{}", pipe));
        say("white", " pipeline for project: ");
        say("magenta", &format!("{}: ", proj));
        try!(client.create_pipeline(&org, &proj, &pipe));
    }
    Ok(())
}

/// Create a Delivery Project with Delivery as SCP (default)
fn create_delivery_project(client: &APIClient,
                           config: &Config) -> Result<(), DeliveryError> {
    let org = try!(config.organization());
    let proj = try!(config.project());
    if client.project_exists(&org, &proj) {
        say("white", "Project ");
        say("magenta", &format!("{} ", proj));
        sayln("white", "already exists.");
    } else {
        say("white", "Creating ");
        say("magenta", "delivery");
        say("white", " project: ");
        say("magenta", &format!("{} ", proj));
        try!(client.create_delivery_project(&org, &proj));
    }
    Ok(())
}

/// Add the `delivery` remote to the local git reposiory and
/// then push local content to the Delivery Server
fn push_project_content_to_delivery(config: &Config, path: &PathBuf) -> Result<(), DeliveryError> {
    let url = try!(config.delivery_git_ssh_url());
    if try!(git::config_repo(&url, path)) {
        sayln("white", "Remote 'delivery' added to git config!");
    } else {
        sayln("white", "Remote named 'delivery' already exists and is correct - not modifying");
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
    try!(create_on_server(&config, scp.clone(), local));
    try!(create_feature_branch(&project_path));
    try!(generate_build_cookbook(skip_build_cookbook, config.generator().ok()));
    try!(generate_custom_delivery_config(config.config_json().ok()));
    try!(trigger_review(config, scp, &no_open, &local));
    Ok(())
}

/// Handle the delivery config generation
///
/// Receives a custom config.json file that will be copy to the current project repo
/// otherwise it will generate the default config.
fn generate_custom_delivery_config(config_json: Option<String>) -> Result<(), DeliveryError> {
    let project_path = try!(root_dir(&utils::cwd()));
    if let Some(json) = config_json {
        let json_path = PathBuf::from(json);
        DeliveryConfig::copy_config_file(&json_path, &project_path)
    } else {
        return Ok(());
    }
}

/// Triggers a delvery review
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
                    sayln("green", "\nYour project is now set up with changes in the add-delivery-config branch!");
                    sayln("green", "To finalize your project, you must submit and accept a Pull Request in github.");

                    // Check to see if the origin remote is set up, and if not, output something useful.
                    let dir = try!(root_dir(&utils::cwd()));
                    let git_remote_result = git::git_command(&["remote"], &dir);
                    match git_remote_result {
                        Ok(git_result) => {
                            if !(git_result.stdout.contains("origin")) {
                                sayln("green", "First, you must add your remote.");
                                sayln("green", "Run this if you want to use ssh:\n");
                                sayln("green", &format!("git remote add origin git@github.com:{}/{}.git\n", s.organization, s.repo_name));
                                sayln("green", "Or this for https:\n");
                                sayln("green", &format!("git remote add origin https://github.com/{}/{}.git\n", s.organization, s.repo_name));
                            }
                            true
                        },
                        Err(_) => false
                    };

                    sayln("green", "Push your project to github by running:\n");
                    sayln("green", "git push origin add-delivery-config\n");
                    sayln("green", "Then log into github via your browser, make a Pull Request, then comment `@delivery approve`.");
                }
            }
        },
        None => try!(cli::review(&pipeline, &false, no_open, &false))
    }
    Ok(())
}

/// Create the feature branch `add-delivery-config`
///
/// This branch is created to start modifying the project repository
/// In the case of a failure, we could roll back fearly easy by checking
/// out master and deleting this feature branch.
fn create_feature_branch(project_path: &PathBuf) -> Result<(), DeliveryError> {
    say("white", "Creating and checking out ");
    say("yellow", "add-delivery-config");
    say("white", " feature branch: ");
    match git::git_command(&["checkout", "-b", "add-delivery-config"], project_path) {
        Ok(_) => {
            sayln("green", "done");
            return Ok(());
        },
        Err(e) => {
            match e.detail.clone() {
                Some(msg) => {
                    if msg.contains("A branch named 'add-delivery-config' already exists") {
                        say("white", "A branch named 'add-delivery-config' already exists, switching to it.\n");
                        try!(git::git_command(&["checkout", "add-delivery-config"], project_path));
                        return Ok(());
                    } else {
                        return Err(e)
                    }
                },
                None => return Err(e)
            }
        }
    }
}

/// Add and commit the generated build-cookbook
fn add_commit_build_cookbook() -> Result<(), DeliveryError> {
    let project_path = try!(root_dir(&utils::cwd()));
    say("white", "Adding and commiting build-cookbook: ");
    // .delivery is probably not yet under version control, so we have to add
    // the whole folder instead of .delivery/build-cookbook.
    try!(git::git_command(&["add", ".delivery"], &project_path));
    try!(git::git_command(&["commit", "-m", "Adds Delivery build cookbook and config"], &project_path));
    sayln("green", "done");
    Ok(())
}

/// Clone a build-cookbook generator if it doesn't exist already on the cache
fn git_clone_build_cookbook_generator(path: &str, url: &str) -> Result<(), DeliveryError> {
    if is_dir(&Path::new(path)) {
        sayln("yellow", &format!("Using cached copy of build-cookbook generator {:?}",
                                 path));
        Ok(())
    } else {
        say("white", "Downloading build-cookbook generator from ");
        sayln("yellow", &format!("{:?}", url));
        git::clone(path, url)
    }
}

/// Custom build-cookbook generation
///
/// This method handles a custom generator which could be:
/// 1) A local path
/// 2) Or a git repo URL
/// TODO) From Supermarket
fn custom_build_cookbook_generator(generator: &Path, path: &Path) -> Result<(), DeliveryError> {
    if generator.has_root() {
        say("white", "Copying custom build-cookbook generator to ");
        sayln("yellow", &format!("{:?}", path));
        try!(utils::copy_recursive(&generator, &path));
    } else {
        try!(git_clone_build_cookbook_generator(&path.to_string_lossy(),
                                                &generator.to_string_lossy()));
    }
    Ok(())
}

/// Default cookbooks generator cache path
fn generator_cache_path() -> Result<PathBuf, DeliveryError> {
    utils::home_dir(&[".delivery/cache/generator-cookbooks"])
}

/// Handles the build-cookbook generation
///
/// This method could receive a custom generator, if it is not provided,
/// we use the default cookbook generator called PCB:
/// => https://github.com/chef-cookbooks/pcb.git
fn generate_build_cookbook(skip_build_cookbook: &bool,
                           generator: Option<String>) -> Result<(), DeliveryError> {
    if *skip_build_cookbook {
        return Ok(())
    }
    sayln("white", "Generating build cookbook skeleton");
    let mut generator_path = try!(generator_cache_path());
    debug!("Cookbook generator cached path: {:?}", generator_path);
    match generator {
        Some(generator_str) => {
            let gen_path = Path::new(&generator_str);
            generator_path.push(gen_path.file_stem().unwrap());
            try!(custom_build_cookbook_generator(&gen_path, &generator_path));
            try!(chef_generate_build_cookbook_from_generator(&generator_path));
        },
        None => {
            let project_path = try!(root_dir(&utils::cwd()));
            let path = project_path.join(".delivery/build-cookbook");
            if path.exists() {
                sayln("red", ".delivery/build-cookbook folder already exists, skipping build cookbook generation.");
                return Ok(());
            } else {
                let dot_delivery = Path::new(".delivery");
                try!(mkdir_recursive(dot_delivery));
                let mut gen = utils::make_command("chef");
                gen.arg("generate")
                    .arg("build-cookbook")
                    .arg(".delivery/build-cookbook")
                    .current_dir(&project_path);
                let output = try!(gen.output());
                try!(handle_chef_generate_cookbook_cmd(output));
                sayln("green", &format!("Build-cookbook generated: {:#?}", gen));
            }
        }
    };
    try!(add_commit_build_cookbook());
    Ok(())
}

/// Generate the build-cookbook using ChefDK generate
fn chef_generate_build_cookbook_from_generator(generator: &Path) -> Result<(), DeliveryError> {
    let project_path = try!(root_dir(&utils::cwd()));
    let dot_delivery = Path::new(".delivery");
    try!(mkdir_recursive(dot_delivery));
    let mut gen = utils::make_command("chef");
    gen.arg("generate")
        .arg("cookbook")
        .arg(".delivery/build-cookbook")
        .arg("-g")
        .arg(generator)
        .current_dir(&project_path);

    debug!("build-cookbook generation with command: {:#?}", gen);
    let output = try!(gen.output());

    debug!("chef-generate-cmd status: {}", output.status);
    try!(handle_chef_generate_cookbook_cmd(output));
    sayln("green", &format!("Build-cookbook generated: {:#?}", gen));
    Ok(())
}

fn handle_chef_generate_cookbook_cmd(output: Output) -> Result<(), DeliveryError> {
    if !output.status.success() {
        return Err(
            DeliveryError {
                kind: Kind::FailedToExecute,
                detail: Some(format!(
                            "Failed to execute chef generate:\n\
                            STDOUT: {}\nSTDERR: {}",
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr)
                        ))
            }
        )
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
