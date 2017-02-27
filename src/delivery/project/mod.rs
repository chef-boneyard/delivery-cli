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

use utils::{self, walk_tree_for_path, mkdir_recursive, cmd_success_or_err};
use utils::path_ext::is_dir;
use errors::{DeliveryError, Kind};
use types::DeliveryResult;
use std::path::{Path, PathBuf};
use http::APIClient;
use git::{self, ReviewResult};
use std::process::Command;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use config::Config;

// README with a brief description of delivery and how to use it. This is added
// to a new project by `delivery init` so we have something to submit as the
// first change.
//
// We load this up as bytes since that's what std::io::Write takes for
// arguments.
static DELIVERY_DOT_MD_CONTENT: &'static [u8] = include_bytes!("DELIVERY.md");

#[derive(Debug, Clone, PartialEq, Eq)]
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
    // Create a new `SourceCodeProvider`. Returns an error result if
    // required configuration values are missing. Expects to find
    // `scp`, `repository`, `scp-organization`, `branch`, and `ssl`.
    pub fn new(scp: &str, repo: &str, org: &str, branch: &str,
               no_ssl: bool) -> DeliveryResult<SourceCodeProvider> {
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

    // Transform the kind Type to str
    pub fn kind_to_fancy_str(&self) -> DeliveryResult<&str> {
        match self.kind {
            Type::Github => Ok("GitHub"),
            Type::Bitbucket => Ok("Bitbucket")
        }
    }

    // Verify if the SCP is configured on the Delivery Server
    pub fn verify_server_config(&self, client: &APIClient) -> DeliveryResult<()> {
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

// Create a Delivery Pipeline.
// Returns true if created, returns false if already exists.
pub fn create_delivery_pipeline(client: &APIClient, org: &str,
                                proj: &str, pipe: &str) -> DeliveryResult<bool> {
    if client.pipeline_exists(org, proj, pipe) {
        return Ok(false)
    } else {
        try!(client.create_pipeline(org, proj, pipe, Some(pipe)));
        return Ok(true)
    }
}

// Create a Delivery Project with Delivery as SCP (default).
// If the project is created, return true.
// If the project already exists, return false
pub fn create_delivery_project(client: &APIClient, org: &str,
                               proj: &str) -> DeliveryResult<bool> {
    if client.project_exists(org, proj) {
        return Ok(false)
    } else {
        try!(client.create_delivery_project(org, proj));
        return Ok(true)
    }
}

pub fn ensure_git_remote_up_to_date(config: &Config) -> DeliveryResult<()> {
    try!(git::create_or_update_delivery_remote(&try!(config.delivery_git_ssh_url()),
                                               &try!(project_path())
    ));
    Ok(())
}

// Push local content to the Delivery Server if no upstream commits.
// Returns true if commits pushed, returns false if upstream commits found.
pub fn push_project_content_to_delivery(pipeline: &str) -> DeliveryResult<bool> {
    if try!(git::server_content(pipeline)) {
        Ok(false)
    } else {
        try!(git::git_push(pipeline));
        Ok(true)
    }
}

// Check to see if the origin remote is set up.
pub fn missing_github_remote() -> DeliveryResult<bool> {
    let git_remote_result = git::git_command(&["remote"], &try!(project_path()));
    match git_remote_result {
        Ok(git_result) => Ok(!git_result.stdout.contains("origin")),
        Err(e) => return Err(e)
    }
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
/// //Stepping into `delivery-cli/src/delivery`
/// let mut delivery_src = env::current_dir().unwrap();
/// delivery_src.push("src/delivery");
///
/// assert_eq!(root, root_dir(&delivery_src.as_path()).unwrap());
/// ```
pub fn root_dir(dir: &Path) -> DeliveryResult<PathBuf> {
    match walk_tree_for_path(&PathBuf::from(&dir), ".git/config") {
        Some(p) => {
           let git_d = p.parent().unwrap();
           let root_d = git_d.parent().unwrap();
           Ok(PathBuf::from(root_d))
        },
        None => Err(DeliveryError{kind: Kind::NoGitConfig,
                                  detail: None})
    }
}

pub fn project_path() -> DeliveryResult<PathBuf> {
    root_dir(&utils::cwd())
}

// Return the project name from the current path
pub fn project_from_cwd() -> DeliveryResult<String> {
    let cwd = try!(self::root_dir(&utils::cwd()));
    Ok(cwd.file_name().unwrap().to_str().unwrap().to_string())
}

// Return the project name or try to extract it from the current path
pub fn project_or_from_cwd(proj: &str) -> DeliveryResult<String> {
    if proj.is_empty() {
        project_from_cwd()
    } else {
        Ok(proj.to_string())
    }
}

// Create the feature branch `add-delivery-config`
//
// This branch is created to start modifying the project repository
// In the case of a failure, we could roll back fearly easy by checking
// out master and deleting this feature branch.
//
// If feature branch created, return true, else return false.
pub fn create_feature_branch_if_missing(project_path: &PathBuf, branch_name: &str) -> DeliveryResult<bool> {
    match git::git_command(&["checkout", "-b", branch_name], project_path) {
        Ok(_) => {
            return Ok(true);
        },
        Err(e) => {
            match e.detail.clone() {
                Some(msg) => {
                    if msg.contains(&format!("A branch named '{}' already exists", branch_name)) {
                       try!(git::git_command(&["checkout", branch_name], project_path));
                        return Ok(false)
                    } else {
                        return Err(e)
                    }
                },
                // Unexpected error, raise.
                None => Err(e)
            }
        }
    }
}

// Add and commit the generated build_cookbook
pub fn add_commit_build_cookbook(custom_config_passed: &bool) -> DeliveryResult<bool> {
    // .delivery is probably not yet under version control, so we have to add
    // the whole folder instead of .delivery/build_cookbook.
    try!(git::git_command(&["add", ".delivery"], &try!(project_path())));

    let mut commit_msg = "Adds Delivery build cookbook".to_string();
    if *custom_config_passed {
        commit_msg = commit_msg + " and config";
    }

    // Commit the changes made in .delivery but detect if nothing has changed,
    // if that is the case, we are Ok() to continue
    match git::git_commit(&commit_msg) {
      Ok(_) => Ok(true),
      Err(DeliveryError{ kind: Kind::EmptyGitCommit, .. }) => Ok(false),
      Err(e) => Err(e)
    }
}

// Create the delivery readme if it doesn't exist already.
pub fn create_delivery_readme() -> DeliveryResult<bool> {
    // NOTE: this isn't guaranteed to be in the project root; however it is only invoked via
    // `delivery init` which makes some assumptions elsewhere that the CWD is the project root.
    if !PathBuf::from("DELIVERY.md").exists() {
        let mut f = try!(File::create("DELIVERY.md"));
        try!(f.write_all(&DELIVERY_DOT_MD_CONTENT));
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn commit_delivery_readme() -> DeliveryResult<()> {
    try!(git::git_command(&["add", "DELIVERY.md"], &try!(project_path())));
    let commit_msg = "New pipeline verification commit".to_string();
    try!(git::git_command(&["commit", "-m", &commit_msg], &try!(project_path())));
    Ok(())
}

pub fn create_dot_delivery() -> &'static Path {
    // TODO: should we be doing some relative pathing here?
    let dot_delivery = Path::new(".delivery");
    fs::create_dir_all(dot_delivery).unwrap();
    dot_delivery
}

pub fn create_build_cookbook<P>(pipeline: &str, path: P) -> DeliveryResult<Command>
        where P: AsRef<Path> {
    let mut command = utils::make_command("chef");
    command.arg("generate")
        .arg("build-cookbook")
        .arg(path.as_ref())
        .arg("--pipeline")
        .arg(pipeline)
        .current_dir(&try!(project_path()));
    let output = command.output()?;
    cmd_success_or_err(&output, Kind::ChefdkGenerateFailed)?;
    Ok(command)
}

#[derive(Debug)]
pub enum CustomCookbookSource {
    Cached,
    Disk,
    Git
}

// Custom build_cookbook generation
//
// This method handles a custom generator which could be:
// 1) A local path
// 2) Or a git repo URL
// TODO) From Supermarket
pub fn download_or_mv_custom_build_cookbook_generator(
        generator: &Path,
        cache_path: &Path) -> DeliveryResult<CustomCookbookSource> {
    try!(mkdir_recursive(cache_path));
    if generator.has_root() {
        try!(utils::copy_recursive(&generator, &cache_path));
        return Ok(CustomCookbookSource::Disk)
    } else {
        let mut cache_generator_path: PathBuf = cache_path.to_path_buf();
        cache_generator_path.push(generator.file_name().unwrap());
        if is_dir(&cache_generator_path) {
            return Ok(CustomCookbookSource::Cached)
        } else {
            let cache_path_str = &cache_generator_path.to_string_lossy();
            let generator_str  = &generator.to_string_lossy();
            try!(git::clone(&cache_path_str, &generator_str));
            return Ok(CustomCookbookSource::Git)
        }
    }
}

// Generate the build_cookbook using ChefDK generate
pub fn chef_generate_build_cookbook_from_generator(
      generator: &Path, project_path: &Path) -> DeliveryResult<Command> {
    let mut command = utils::make_command("chef");
    command.arg("generate")
        .arg("build-cookbook")
        .arg(".")
        .arg("-g")
        .arg(generator)
        .current_dir(&project_path);

    let output = command.output()?;
    cmd_success_or_err(&output, Kind::ChefdkGenerateFailed)?;
    Ok(command)
}

// Default cookbooks generator cache path
pub fn generator_cache_path() -> DeliveryResult<PathBuf> {
    utils::home_dir(&[".delivery/cache/generator-cookbooks"])
}

pub fn review(target: &str, head: &str) -> DeliveryResult<ReviewResult> {
    if target == head {
        Err(DeliveryError{ kind: Kind::CannotReviewSameBranch, detail: None })
    } else {
        Ok(try!(git::git_push_review(head, target)))
    }
}

pub fn handle_review_result(review: &ReviewResult,
                            no_open: &bool) -> DeliveryResult<Option<String>> {
    match review.url {
        Some(ref url) => {
            if !no_open {
                try!(utils::open::item(&url));
            }
            Ok(Some(url.clone()))
        },
        None => Ok(None)
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
