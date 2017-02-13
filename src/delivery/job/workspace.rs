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

use errors::{DeliveryError, Kind};
use types::DeliveryResult;
use git;
use serde_json;
use delivery_config::{DeliveryConfig, BuildCookbookLocation};
use job::dna::{Top, DNA, WorkspaceCompat};
use job::change::{Change, BuilderCompat};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use utils;
use utils::path_to_string;
use utils::path_join_many::PathJoinMany;
use utils::path_ext::{is_file, is_dir};
use std::error;
use config::Config;

pub struct Workspace {
    pub root: PathBuf,
    pub chef: PathBuf,
    pub cache: PathBuf,
    pub repo: PathBuf,
    pub ssh_wrapper: PathBuf
}

#[derive(Debug)]
pub enum Privilege {
    Drop,
    NoDrop
}

// Here's the config.rb we render for the chef-zero runs.
static CONFIG_RB: &'static str = r#"
file_cache_path File.expand_path(File.join(File.dirname(__FILE__), '..', 'cache'))
cache_type 'BasicFile'
cache_options(:path => File.join(file_cache_path, 'checksums'))
cookbook_path File.expand_path(File.join(File.dirname(__FILE__), 'cookbooks'))
file_backup_path File.expand_path(File.join(File.dirname(__FILE__), '..', 'cache', 'job-backup'))
Ohai::Config[:disabled_plugins] = [ :Passwd ]
if ENV['DELIVERY_BUILD_SETUP'] == 'FALSE'
  lockfile File.join(file_cache_path, 'chef-client-running.pid')
else
  if File.exists?('/var/chef/cache/chef-client-running.pid')
    lockfile '/var/chef/cache/chef-client-running.pid'
  else
    lockfile File.join(file_cache_path, 'chef-client-running.pid')
  end
end
"#;

impl Workspace {
    pub fn new(root: &PathBuf) -> Workspace {
        Workspace{
            root: root.clone(),
            chef: root.join("chef"),
            cache: root.join("cache"),
            repo: root.join("repo"),
            ssh_wrapper: root.join("bin").join("git_ssh")
        }
    }

    // Build the workspace tree on the build-node
    pub fn build(&self) -> Result<(), DeliveryError> {
        try!(self.clean_chef_nodes());
        try!(utils::mkdir_recursive(&self.root));
        try!(utils::mkdir_recursive(&self.chef.join("nodes")));
        try!(utils::mkdir_recursive(&self.chef.join("cookbooks")));
        try!(utils::mkdir_recursive(&self.cache));
        try!(utils::mkdir_recursive(&self.repo));
        Ok(())
    }

    // Clean the workspace::chef/nodes directory
    //
    // We have to clean the `nodes/` directory since `chef-zero`
    // will merge the old attributes with the new ones of the
    // coming change.
    pub fn clean_chef_nodes(&self) -> Result<(), DeliveryError> {
       try!(utils::remove_recursive(&self.chef.join("nodes")));
       Ok(())
    }

    fn reset_repo(&self, git_ref: &str) -> Result<(), DeliveryError> {
        try!(git::git_command(&["reset", "--hard", git_ref], &self.repo));
        try!(git::git_command(&["clean", "-x", "-f", "-d", "-q"], &self.repo));
        Ok(())
    }

    fn setup_build_cookbook_from_path(&self, path: &PathBuf) -> DeliveryResult<()> {
        utils::copy_recursive(path, &self.chef.join("build_cookbook"))
    }

    fn setup_build_cookbook_from_git(&self,
                        config: &DeliveryConfig) -> DeliveryResult<()> {
        let git_url = try!(config.build_cookbook_get("git"));
        let branch = config.build_cookbook_get("branch").unwrap_or("master".to_owned());
        let build_cookbook_path = &self.chef.join("build_cookbook");
        try!(git::git_command(&["clone", &git_url,
                                &path_to_string(build_cookbook_path)],
                              &self.chef));
        try!(git::git_command(&["checkout", &branch], build_cookbook_path));
        Ok(())
    }

    // This will need a windows implementation, and probably won't work on non-gnu tar systems
    // either.
    fn setup_build_cookbook_from_supermarket(&self,
                        config: &DeliveryConfig) -> DeliveryResult<()> {
        let name = try!(config.build_cookbook_name());
        let site = config.build_cookbook_get("site")
                        .unwrap_or("https://supermarket.chef.io".to_owned());
        let result = try!(utils::make_command("knife")
             .arg("supermarket")
             .arg("download")
             .arg(&name)
             .arg("-m")
             .arg(&site)
             .arg("-f")
             .arg(&path_to_string(&self.chef.join("build_cookbook.tgz")))
             .current_dir(&self.root)
             .output());
        if ! result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            let error = String::from_utf8_lossy(&result.stderr);
            return Err(DeliveryError{
                kind: Kind::SupermarketFailed,
                detail: Some(
                    format!("Failed 'knife supermarket download'\nOUT: {}\nERR: {}\nSite: {}",
                    &output, &error, &site).to_string()
                )
            });
        }
        let tar_result = try!(utils::make_command("tar")
             .arg("zxf")
             .arg(&path_to_string(&self.chef.join("build_cookbook.tgz")))
             .current_dir(&self.chef)
             .output());
        if ! tar_result.status.success() {
            let output = String::from_utf8_lossy(&tar_result.stdout);
            let error = String::from_utf8_lossy(&tar_result.stderr);
            return Err(DeliveryError{
                kind: Kind::TarFailed,
                detail: Some(
                    format!("Failed 'tar zxf'\nOUT: {}\nERR: {}",
                    &output, &error).to_string()
                )
            });
        }
        let mv_result = try!(utils::make_command("mv")
             .arg(&path_to_string(&self.chef.join(name)))
             .arg(&path_to_string(&self.chef.join("build_cookbook")))
             .current_dir(&self.chef)
             .output());
        if ! mv_result.status.success() {
            let output = String::from_utf8_lossy(&mv_result.stdout);
            let error = String::from_utf8_lossy(&mv_result.stderr);
            return Err(DeliveryError{
                kind: Kind::MoveFailed,
                detail: Some(
                    format!("Failed 'mv'\nOUT: {}\nERR: {}",
                    &output, &error).to_string()
                )
            });
        }
        Ok(())
    }

    fn setup_build_cookbook_from_chef_server(&self, name: &str) -> DeliveryResult<()> {
        try!(utils::mkdir_recursive(&self.chef.join("tmp_cookbook")));
        let result = try!(utils::make_command("knife")
                          .arg("download")
                          .arg(&format!("/cookbooks/{}", &name))
                          .arg("--chef-repo-path")
                          .arg(&path_to_string(&self.chef.join("tmp_cookbook")))
                          .current_dir(&self.root)
                          .output());
        if ! result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            let error = String::from_utf8_lossy(&result.stderr);
            return Err(DeliveryError{
                kind: Kind::ChefServerFailed,
                 detail: Some(
                    format!("Failed 'knife cookbook download'\nOUT: {}\nERR: {}",
                    &output, &error).to_string()
                )
            });
        }
        let mv_result = try!(utils::make_command("mv")
                             .arg(&path_to_string(&self.chef.join_many(&["tmp_cookbook",
                                                                         "cookbooks", &name])))
                             .arg(&path_to_string(&self.chef.join("build_cookbook")))
                             .current_dir(&self.chef)
                             .output());
        if ! mv_result.status.success() {
            let output = String::from_utf8_lossy(&mv_result.stdout);
            let error = String::from_utf8_lossy(&mv_result.stderr);
            return Err(DeliveryError{
                kind: Kind::MoveFailed,
                detail: Some(
                    format!("Failed 'mv'\nOUT: {}\nERR: {}",
                    &output, &error).to_string()
                )
            });
        }
        Ok(())
    }

    fn setup_build_cookbook_from_workflow(&self,
                                          config: &DeliveryConfig,
                                          toml_config: &Config) -> DeliveryResult<()> {
        let name = try!(config.build_cookbook_name());
        let ent = try!(config.build_cookbook_get("enterprise"));
        let org = try!(config.build_cookbook_get("organization"));
        let build_cookbook_config = toml_config.clone().set_enterprise(&ent)
                                                       .set_organization(&org)
                                                       .set_project(&name);
        let url = try!(build_cookbook_config.delivery_git_ssh_url());
        try!(git::git_command(
                &["clone", &url, self.chef.join("build_cookbook").to_str().unwrap()],
                &self.chef
        ));
        Ok(())
    }

    fn setup_build_cookbook(&self, toml_config: &Config,
                            config: &DeliveryConfig) -> DeliveryResult<()> {
        match try!(config.build_cookbook_location()) {
            BuildCookbookLocation::Local => {
                let ab_path = self.repo.join(try!(config.build_cookbook_get("path")));
                self.setup_build_cookbook_from_path(&ab_path)
            },
            BuildCookbookLocation::Git => {
                self.setup_build_cookbook_from_git(config)
            },
            BuildCookbookLocation::Supermarket => {
                self.setup_build_cookbook_from_supermarket(config)
            },
            BuildCookbookLocation::Workflow => {
                self.setup_build_cookbook_from_workflow(config, toml_config)
            },
            BuildCookbookLocation::ChefServer => {
                let name = try!(config.build_cookbook_name());
                self.setup_build_cookbook_from_chef_server(&name)
            },
        }
    }

    fn berks_vendor(&self, bc_name: &str) -> DeliveryResult<()> {
        try!(utils::remove_recursive(&self.chef.join("cookbooks")));
        if is_file(&self.chef.join_many(&["build_cookbook", "Berksfile"])) {
            debug!("Running 'berks vendor cookbooks' inside the build_cookbooks");
            let mut command = utils::make_command("berks");
            command.arg("vendor");
            command.arg(&self.chef.join("cookbooks"));
            command.current_dir(&self.chef.join("build_cookbook"));
            let output = match command.output() {
                Ok(o) => o,
                Err(e) => {
                    let d = format!("failed to execute 'berks vendor {}' from '{}': {}",
                                    &path_to_string(&self.chef.join("cookbooks")),
                                    &path_to_string(&self.chef.join("build_cookbook")),
                                    error::Error::description(&e));
                    return Err(DeliveryError{ kind: Kind::FailedToExecute,
                                                      detail: Some(d)}) },
            };
            if !output.status.success() {
                return Err(DeliveryError{
                    kind: Kind::BerksFailed,
                    detail: Some(
                        format!("STDOUT: {}\nSTDERR: {}\n",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr))
                    )
                });
            }
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            debug!("berks vendor stdout: {}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            debug!("berks vendor stderr: {}", stderr);
        } else {
            debug!("No Berksfile found; simply moving the cookbook");
            try!(utils::mkdir_recursive(&self.chef.join("cookbooks")));
            let mv_result = try!(Command::new("mv")
                                 .arg(&path_to_string(&self.chef.join("build_cookbook")))
                                 .arg(&path_to_string(&self.chef.join_many(&["cookbooks", bc_name])))
                                 .current_dir(&self.chef)
                                 .output());
            if ! mv_result.status.success() {
                let output = String::from_utf8_lossy(&mv_result.stdout);
                let error = String::from_utf8_lossy(&mv_result.stderr);
                return Err(DeliveryError{
                    kind: Kind::MoveFailed,
                    detail: Some(
                        format!("Failed 'mv'\nOUT: {}\nERR: {}",
                        &output, &error).to_string()
                    )
                });
            }
        }
        Ok(())
    }

    /// This sets permissions in the workspace repo and cache directories.
    pub fn set_drop_permissions(&self) -> Result<(), DeliveryError> {
        let paths_to_chown = &[&self.repo,
                               &self.chef.join("cookbooks"),
                               &self.chef.join("nodes"),
                               &self.cache];
        utils::chown_all("dbuild:dbuild", paths_to_chown)
    }

    pub fn run_job(&self, phase_arg: &str,
                    drop_privilege: &Privilege,
                    local_change: &bool) -> DeliveryResult<()> {
        let config = try!(DeliveryConfig::load_config(&self.repo));
        let bc_name = try!(config.build_cookbook_name());
        let run_list = {
            let phases: Vec<String> = phase_arg.split(" ")
                .map(|p| format!("{}::{}", bc_name, p)).collect();
            phases.join(",")
        };
        let mut command = utils::make_command("chef-client");
        command.arg("-z").arg("--force-formatter");
        try!(self.handle_privilege_drop(drop_privilege, &mut command));
        if ! local_change {
          command.env("HOME", &path_to_string(&self.cache));
        }
        command.arg("-j")
            .arg(&path_to_string(&self.chef.join("dna.json")))
            .arg("-c")
            .arg(&path_to_string(&self.chef.join("config.rb")))
            .arg("-r")
            .arg(run_list)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(&self.repo);
        match phase_arg {
            "default" => command.env("DELIVERY_BUILD_SETUP", "TRUE"),
            _ => command.env("DELIVERY_BUILD_SETUP", "FALSE")
        };
        debug!("Job Command: {:?}", command);
        let output = match command.output() {
            Ok(o) => o,
            Err(e) => {
                return Err(DeliveryError{
                    kind: Kind::FailedToExecute,
                    detail: Some(
                        format!("failed to execute chef-client: {}",
                        error::Error::description(&e))
                    )
                })
            },
        };
        if !output.status.success() {
            return Err(DeliveryError{
                kind: Kind::ChefFailed,
                detail: Some(
                    format!("STDOUT: {}\nSTDERR: {}\n",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr))
                )
            });
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn handle_privilege_drop(&self, privilege: &Privilege,
                             cmd: &mut Command) -> Result<(), DeliveryError> {
        match privilege {
            &Privilege::Drop => {
                try!(self.set_drop_permissions());
                cmd.arg("--user")
                    .arg("dbuild")
                    .arg("--group")
                    .arg("dbuild");
            },
            _ => {}
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    #[allow(unused_variables)]
    fn handle_privilege_drop(&self, privilege: &Privilege,
                             cmd: &mut Command) -> Result<(), DeliveryError> {
        Ok(())
    }

    pub fn setup_chef_for_job(&self,
                              toml_config: &Config, change: Change,
                              ws_path: &PathBuf) -> Result<(), DeliveryError> {
        let config_rb_path = &self.chef.join("config.rb");
        debug!("Writing content of chef/config.rb");
        let mut config_rb = try!(File::create(config_rb_path));
        try!(utils::chmod(config_rb_path, "0644"));
        try!(config_rb.write_all(CONFIG_RB.as_bytes()));
        let config = try!(DeliveryConfig::load_config(&self.repo));
        debug!("Setting up the build_cookbook");
        try!(self.setup_build_cookbook(toml_config, &config));
        let build_cb_name = try!(config.build_cookbook_name());
        try!(self.berks_vendor(&build_cb_name));
        let workspace_data = WorkspaceCompat{
            root: path_to_string(&self.root),
            chef: path_to_string(&self.chef),
            cache: path_to_string(&self.cache),
            repo: path_to_string(&self.repo),
            ssh_wrapper: path_to_string(&self.ssh_wrapper),
        };
        let top = Top{
            workspace_path: path_to_string(ws_path),
            workspace: workspace_data,
            change: change,
            config: config
        };
        let compat = BuilderCompat{
            workspace: path_to_string(&self.root),
            repo: path_to_string(&self.repo),
            cache: path_to_string(&self.cache),
            build_id: "deprecated".to_string(),
            build_user: "dbuild".to_string()
        };
        let dna = DNA{
            delivery: top,
            delivery_builder: compat
        };
        debug!("Writing content of chef/dna.json");
        let dna_json_path = &self.chef.join("dna.json");
        let mut dna_json = try!(File::create(dna_json_path));
        try!(utils::chmod(dna_json_path, "0644"));
        let data = try!(serde_json::to_string(&dna));
        try!(dna_json.write_all(data.as_bytes()));
        Ok(())
    }

    pub fn setup_repo_for_change(&self, git_url: &str, change_branch: &str,
                                 pipeline: &str, sha: &str) -> DeliveryResult<()> {
        if ! is_dir(&self.repo.join(".git")) {
            try!(git::git_command(&["clone", git_url, "."], &self.repo));
        }
        try!(git::git_command(&["fetch", "origin"], &self.repo));
        try!(self.reset_repo("HEAD"));
        try!(git::git_command(&["checkout", pipeline], &self.repo));
        try!(self.reset_repo(&format!("remotes/origin/{}", pipeline)));
        if sha.is_empty() {
            try!(git::git_command(&["fetch", "origin", change_branch], &self.repo));
            try!(git::git_command(&["merge", "--strategy", "resolve", "FETCH_HEAD"], &self.repo));
        } else {
            try!(self.reset_repo(sha))
        }
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use utils;
    use utils::path_ext::{is_dir, is_file};
    use std::path::PathBuf;

    #[test]
    fn test_workspace_new() {
        let root = PathBuf::from("clown");
        let w = Workspace::new(&root);
        assert_eq!(w.root, root);
        assert_eq!(w.chef, root.join("chef"));
        assert_eq!(w.cache, root.join("cache"));
        assert_eq!(w.repo, root.join("repo"));
    }

    #[test]
    fn test_workspace_build() {
        let root = PathBuf::from("/tmp/cli-workspace-build");
        let w = Workspace::new(&root);
        assert!(w.build().is_ok(), "The workspace build process failed");
        assert!(is_dir(&w.root));
        assert!(is_dir(&w.chef));
        assert!(is_dir(&w.chef.join("nodes")));
        assert!(is_dir(&w.cache));
        assert!(is_dir(&w.repo));
        // Remove temp cli workspace
        utils::remove_recursive(&root).unwrap();
    }

    #[test]
    fn test_workspace_build_and_clean_chef_nodes() {
        let root = PathBuf::from("/tmp/cli-workspace-clean");
        let w = Workspace::new(&root);
        assert!(w.build().is_ok(), "The workspace build process failed");
        // This is an empty workspace, lets lay down a file
        // inside chef/nodes and test it exists, then after
        // running build() again it shouldn't exist anymore.
        let nfile = w.chef.join("nodes").join("test.node");
        let _ = File::create(nfile.clone());
        assert!(is_file(&nfile));
        assert!(w.build().is_ok());
        assert_eq!(false, is_file(&nfile));
        // Remove temp cli workspace
        utils::remove_recursive(&root).unwrap();
    }
}
