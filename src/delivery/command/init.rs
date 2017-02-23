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
// Takes in fully parsed and defaulted init clap args,
// executes init codeflow, handles user actionable errors, as well as UI output.
//
// Returns an integer exit code, handling all errors it knows how to
// and panicing on unexpected errors.

use std;
use fips;
use cli::init::InitClapOptions;
use delivery_config::DeliveryConfig;
use config::Config;
use std::path::{Path, PathBuf};
use project;
use git;
use utils;
use utils::say::{say, sayln};
use std::io;
use http::APIClient;
use errors::{Kind, DeliveryError};
use types::{DeliveryResult, ExitCode};
use hyper::status::StatusCode;
use command::Command;

pub struct InitCommand<'n> {
    pub options: &'n InitClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for InitCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        if !self.options.local {
            try!(project::ensure_git_remote_up_to_date(&self.config));
            try!(fips::setup_and_start_stunnel_if_fips_mode(&self.config, child_processes));
        }
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");

        let branch = try!(self.config.pipeline());

        if !self.options.github_org_name.is_empty()
            && !self.options.bitbucket_project_key.is_empty() {
                sayln("red", "\nPlease specify just one Source Code Provider: \
                              delivery (default), github or bitbucket.");
                return Ok(1)
            }

        let scp = if !self.options.github_org_name.is_empty() {
            Some(
                try!(project::SourceCodeProvider::new("github", &self.options.repo_name,
                                                      &self.options.github_org_name, &branch,
                                                      self.options.no_v_ssl))
            )
        } else if !self.options.bitbucket_project_key.is_empty() {
            Some(
                try!(project::SourceCodeProvider::new("bitbucket", &self.options.repo_name,
                                                      &self.options.bitbucket_project_key,
                                                      &branch, true))
            )
        } else {
            None
        };

        // Initalize the repo.
        let project_path = try!(project::project_path());
        project::create_dot_delivery();

        if !self.options.local {
            try!(create_on_server(&self.config, scp.clone()))
        }


        // Generate build cookbook, either custom or default.
        let custom_build_cookbook_generated = if !self.options.skip_build_cookbook {
            try!(generate_build_cookbook(&self.config))
        } else {
            false
        };

        // Generate delivery config if passed
        let custom_config_passed = try!(
            generate_delivery_config(self.config.config_json().ok())
        );

        // Verify that the project has a config file
        let config_path = project_path.join(".delivery/config.json");
        if !config_path.exists() {
            // Custom error handling for missing config file.
            if custom_build_cookbook_generated && !custom_config_passed {
                sayln("red", "\nYou used a custom build cookbook generator, but \
                              .delivery/config.json was not created.");
                sayln("red", "Please update your generator to create a valid \
                              .delivery/config.json or pass in a custom config.");
                return Ok(1)
            } else {
                let msg = "Missing .delivery/config.json file.\nPlease use a \
                           custom build cookbook generator that creates this \
                           file or pass in a custom config.".to_string();
                return Err(DeliveryError{
                    kind: Kind::MissingConfigFile,
                    detail: Some(msg)
                });
            }
        }

        // If nothing custom was requested, then `chef generate build_cookbook`
        // will put the commits in the initialize-delivery-pipeline branch, otherwise,
        // if a custom build cookbook was generated or a custom config was passed,
        // commits will land in the add-delivery-config branch.
        let branch_name;
        let mut review_needed = false;
        if custom_build_cookbook_generated || custom_config_passed {
            branch_name = "add-delivery-config";
            review_needed = true;
            sayln("cyan", "Committing unmerged Delivery content and submitting for review...");
            if !try!(project::create_feature_branch_if_missing(&project_path, branch_name)) {
                sayln("white", &format!("  Skipping: A branch named '{}' already exists, \
                                         switching to it.", branch_name))
            } else {
                sayln("green", &format!("  Feature branch named '{}' created.", branch_name))
            }

            if custom_build_cookbook_generated {
                if try!(project::add_commit_build_cookbook(&custom_config_passed)) {
                    sayln("green", "  Custom build cookbook committed to feature branch.")
                } else {
                    sayln("white", "  Skipping: Build cookbook was not modified, no need to commit.");
                }
            }

            // project::add_commit_build_cookbook will commit the custom config for us,
            // so if a custom build cookbook was passed, the delivery config was already committed.
            if custom_config_passed && !custom_build_cookbook_generated {
                if try!(DeliveryConfig::git_add_commit_config(&project_path)) {
                    sayln("green", "  Custom delivery config committed to feature branch.")
                } else {
                    sayln("white", "  Skipping: Delivery config was not modified, no need to commit.");
                }
            }
        } else {
            branch_name = "initialize-delivery-pipeline";
            // Create a commit to send to review.
            sayln("cyan", "Creating and committing DELIVERY.md readme...");
            if !try!(project::create_feature_branch_if_missing(&project_path, branch_name)) {
                sayln("white", &format!("  Skipping: A branch named '{}' already exists, \
                                         switching to it.", branch_name))
            } else {
                sayln("green", &format!("  Feature branch named '{}' created.", branch_name))
            }

            // Create and commit DELIVERY.md readme if it doesn't exist.
            if try!(project::create_delivery_readme()) {
                review_needed = true;
                sayln("green", "  DELIVERY.md created.");
                try!(project::commit_delivery_readme());
                sayln("green", &format!("  DELIVERY.md committed in branch '{}'.", branch_name))
            } else {
                sayln("white", "  Skipping: DELIVERY.md already exists, no need to create or commit.");
            }

        }

        // Trigger review if there were any custom commits to review.
        if !self.options.local {
            if review_needed {
                sayln("cyan", &format!("Submitting feature branch '{}' for review...", branch_name));
                try!(trigger_review(self.config, scp, &self.options.no_open));
            } else {
                sayln("white", "  Skipping: All changes have already be submitted for review, skipping.");
            }
        } else {
            sayln("white", " Skipping:  You passed --local, skipping review submission.");
        }

        sayln("green", "\nYour new Delivery project is ready!");
        Ok(0)
    }
}

// Create a Delivery Project
//
// This method will create a Delivery Project depending on the SCP that we specify,
// either a Github, Bitbucket or Delivery (default). It also creates a pipeline,
// adds the `delivery` remote and push the content of the local repo to the Server.
fn create_on_server(config: &Config,
                    scp: Option<project::SourceCodeProvider>) -> DeliveryResult<()> {
    let client = try!(APIClient::from_config(config));
    let org = try!(config.organization());
    let proj = try!(config.project());
    let pipe = try!(config.pipeline());

    match scp {
        // If the user requested a custom scp
        Some(scp_config) => {
            // TODO: actually handle this error
            try!(scp_config.verify_server_config(&client));
            try!(compare_directory_name(&scp_config.repo_name));
            let fancy_kind = try!(scp_config.kind_to_fancy_str());
            let response: StatusCode;

            sayln("cyan", &format!("Creating {} backed Delivery project...", fancy_kind));
            match scp_config.kind {
                project::Type::Bitbucket => {
                    response = try!(client.create_bitbucket_project(
                        &org, &proj, &scp_config.repo_name,
                        &scp_config.organization, &scp_config.branch));
                },
                project::Type::Github => {
                    response = try!(client.create_github_project(&org, &proj, &scp_config.repo_name,
                                                                 &scp_config.organization, &scp_config.branch,
                                                                 scp_config.verify_ssl));
                }
            }

            match response {
                StatusCode::Conflict => {
                    sayln("white", &format!("  Skipping: {} backed Delivery project named {} \
                                             already exists.", fancy_kind, proj));

                },
                _ => {
                    sayln("green", &format!("  {} backed Delivery project named {} \
                                             created.", fancy_kind, proj));
                }
            }
            try!(push_project_content_to_delivery(&pipe));
        },
        // If the user isn't using an scp, just delivery itself.
        None => {
            // Create delivery project on server unless it already exists.
            sayln("cyan", "Creating Delivery project...");
            if try!(project::create_delivery_project(&client, &org, &proj)) {
                sayln("green", &format!("  Delivery project named {} was created.", proj));
            } else {
                sayln("white",
                      &format!("  Skipping: Delivery project named {} already exists.", proj));
            }
            try!(push_project_content_to_delivery(&pipe));
            try!(create_delivery_pipeline(&client, &org, &proj, &pipe));
        }
    }
    Ok(())
}

// Push content to Delivery if no upstream commits.
fn push_project_content_to_delivery(pipeline: &str) -> DeliveryResult<()> {
    sayln("cyan", "Pushing initial git history...");
    if !try!(project::push_project_content_to_delivery(&pipeline)) {
        sayln("white", &format!("  Skipping: Found commits on remote for pipeline {}, \
                                 not pushing local commits.", pipeline))
    } else {
        sayln("green", &format!("  No git history found for pipeline {}, \
                                 pushing local commits from branch {}.", pipeline, pipeline))
    }
    Ok(())
}

// Create Delivery pipeline unless it already exists.
fn create_delivery_pipeline(client: &APIClient, org: &str,
                            proj: &str, pipe: &str) -> DeliveryResult<()> {
    sayln("cyan", "Creating pipeline on Delivery server...");
    if try!(project::create_delivery_pipeline(client, org, proj, pipe)) {
        sayln("green", &format!("  Created Delivery pipeline {} for project {}.",
                                pipe, proj))
    } else {
        sayln("white", &format!("  Skipping: Delivery pipeline \
                                 named {} already exists for project {}.", pipe, proj))
    }
    Ok(())
}

// Handles the build_cookbook generation
//
// Use the provided custom generator, if it is not provided we use the default
// build_cookbook generator from the ChefDK.
//
// Returns true if a CUSTOM build cookbook was generated, else it returns false.
fn generate_build_cookbook(config: &Config) -> DeliveryResult<bool> {
    let cache_path = try!(project::generator_cache_path());
    let project_path = try!(project::project_path());
    match config.generator().ok() {
        Some(generator_str) => {
            sayln("cyan", "Generating custom build cookbook...");
            generate_custom_build_cookbook(generator_str, cache_path, project_path)
        },
        // Default build cookbook
        None => {
            sayln("cyan", "Generating default build cookbook...");
            if try!(project::project_path()).join(".delivery/build_cookbook").exists() {
                sayln("white", "  Skipping: build cookbook already exists at \
                                .delivery/build_cookbook.");
                Ok(false)
            } else {
                let pipeline = try!(config.pipeline());
                try!(project::create_default_build_cookbook(&pipeline));
                sayln("green", "  Build cookbook generated at .delivery/build_cookbook.");
                try!(git::git_push(&pipeline));
                sayln("green",
                      &format!("  Build cookbook committed to git and pushed to pipeline named {}.", pipeline));
                Ok(false)
            }
        }
    }
}

fn generate_custom_build_cookbook(generator_str: String,
                                  cache_path: PathBuf,
                                  project_path: PathBuf) -> DeliveryResult<bool> {
    let gen_path = Path::new(&generator_str);
    let mut generator_path = cache_path.clone();
    generator_path.push(gen_path.file_stem().unwrap());
    match try!(project::download_or_mv_custom_build_cookbook_generator(&gen_path, &cache_path)) {
        project::CustomCookbookSource::Disk => {
            sayln("green", "  Copying custom build cookbook generator to the cache.")
        },
        project::CustomCookbookSource::Cached => {
            sayln("white", "  Skipping: Using cached copy of custom build cookbook generator.")
        },
        project::CustomCookbookSource::Git => {
            sayln("green", &format!("  Downloading build_cookbook generator from {}.", generator_str))
        }
    }

    try!(project::chef_generate_build_cookbook_from_generator(&generator_path, &project_path));
    sayln("green", "  Custom build cookbook generated at .delivery/build_cookbook.");
    return Ok(true)
}

fn generate_delivery_config(config_json: Option<String>) -> DeliveryResult<bool> {
    if let Some(json) = config_json {
        sayln("cyan", "Copying custom Delivery config...");
        let proj_path = try!(project::project_path());
        let json_path = PathBuf::from(&json);

        // Create config
        match try!(DeliveryConfig::copy_config_file(&json_path, &proj_path)) {
            Some(_) => {
                sayln("green", &format!("  Custom Delivery config copied \
                                         from {} to .delivery/config.json.", &json));
                Ok(true)
            },
            None => {
                sayln("white", &format!("  Skipped: Content of custom config passed from {} exactly \
                                         matches existing .delivery/config.json.", &json));
                Ok(false)
            }
        }
    } else {
        Ok(false)
    }
}

// Triggers an delivery review.
fn trigger_review(config: &Config, scp: Option<project::SourceCodeProvider>,
                  no_open: &bool) -> DeliveryResult<()> {
    let pipeline = try!(config.pipeline());
    let head = try!(git::get_head());

    // We now trigger a review for every single project type
    let review = try!(project::review(&pipeline, &head));
    match project::handle_review_result(&review, no_open) {
        Ok(_) => (),
        Err(_) => {
            sayln("yellow", "  We could not open the review in the browser for you.");
            sayln("yellow", "  Make sure there is a program that can open HTML files in your path \
                             or pass --no-open to bypass attempting to open this review in a browser.");
        }
    }
    match scp {
        Some(s) => sayln("green", &format!("  Review submitted to Delivery with {} \
                                            intergration enabled.", try!(s.kind_to_fancy_str()))),
        None => sayln("green", "  Review submitted to Delivery.")
    }
    Ok(())
}

// Compare that the directory name is the same as the repo-name
// provided by the user, if not show a WARN message
fn compare_directory_name(repo_name: &str) -> DeliveryResult<()> {
    let c_dir = utils::cwd();
    if !c_dir.ends_with(repo_name) {
        let mut answer = String::new();
        let project_name = try!(project::project_from_cwd());
        sayln("yellow", &format!(
            "WARN: This project will be named '{}', but the repository name is '{}'.",
            project_name, repo_name));
        say("yellow", "Are you sure this is what you want? y/n: ");
        try!(io::stdin().read_line(&mut answer));
        debug!("You answered '{}'", answer.trim());
        if answer.trim() != "y" {
            let msg = "\nTo match the project and the repository name you can:\n  1) \
                       Create a directory with the same name as the repository.\n  2) \
                       Clone or download the content of the repository inside.\n  3) \
                       Run the 'delivery init' command within the new directory.".to_string();
            return Err(DeliveryError{
                kind: Kind::ProjectSCPNameMismatch,
                detail: Some(msg)
            });
        }
    }
    Ok(())
}

