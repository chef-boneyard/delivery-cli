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
use git;
use rustc_serialize::{Encodable, Encoder};
use rustc_serialize::json::{self, Json};
use job::dna::{Top, DNA, WorkspaceCompat};
use job::change::{Change, BuilderCompat};
use job;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use utils;
use utils::path_join_many::PathJoinMany;
use std::error;

#[derive(RustcDecodable, Debug)]
pub struct Workspace {
    pub root: PathBuf,
    pub chef: PathBuf,
    pub cache: PathBuf,
    pub repo: PathBuf
}

#[derive(Debug)]
pub enum Privilege {
    Drop,
    NoDrop
}

// We want this to encode as strings, not as vectors of bytes.
// It's cool - I accept we'll be lossy if its not a utf8 string.
impl Encodable for Workspace {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("Workspace", 0, |encoder| {
            try!(encoder.emit_struct_field( "root", 0usize, |encoder| self.root.to_str().unwrap().encode(encoder)));
            try!(encoder.emit_struct_field( "chef", 1usize, |encoder| self.chef.to_str().unwrap().encode(encoder)));
            try!(encoder.emit_struct_field( "cache", 2usize, |encoder| self.cache.to_str().unwrap().encode(encoder)));
            try!(encoder.emit_struct_field( "repo", 3usize, |encoder| self.repo.to_str().unwrap().encode(encoder)));
            Ok(())
        })
    }
}

impl Workspace {
    pub fn new(root: &PathBuf) -> Workspace {
        Workspace{
            root: root.clone(),
            chef: root.join("chef"),
            cache: root.join("cache"),
            repo: root.join("repo")
        }
    }

    pub fn build(&self) -> Result<(), DeliveryError> {
        try!(utils::mkdir_recursive(&self.root));
        try!(utils::mkdir_recursive(&self.chef));
        try!(utils::mkdir_recursive(&self.cache));
        try!(utils::mkdir_recursive(&self.repo));
        Ok(())
    }

    fn reset_repo(&self, git_ref: &str) -> Result<(), DeliveryError> {
        try!(git::git_command(&["reset", "--hard", git_ref], &self.repo));
        try!(git::git_command(&["clean", "-x", "-f", "-d", "-q"], &self.repo));
        Ok(())
    }

    fn setup_build_cookbook_from_path(&self, path: &PathBuf) -> Result<(), DeliveryError> {
        utils::copy_recursive(path, &self.chef.join("build_cookbook"))
    }

    fn setup_build_cookbook_from_git(&self, build_cookbook: &Json, git_url: &str) -> Result<(), DeliveryError> {
        let branch = match build_cookbook.find("branch") {
            Some(b) => try!(b.as_string().ok_or(DeliveryError{
                kind: Kind::ExpectedJsonString,
                detail: Some("Expected 'branch' value to be a string".to_string())
            })),
            None => "master"
        };
        try!(git::git_command(&["clone", git_url, self.chef.join("build_cookbook").to_str().unwrap()], &self.chef));
        try!(git::git_command(&["checkout", &branch], &self.chef.join("build_cookbook")));
        Ok(())
    }

    // This will need a windows implementation, and probably won't work on non-gnu tar systems
    // either.
    fn setup_build_cookbook_from_supermarket(&self, build_cookbook: &Json) -> Result<(), DeliveryError> {
        let is_name = build_cookbook.find("name");
        if is_name.is_some() {
            let name = match is_name.unwrap().as_string() {
                Some(n) => n,
                None => return Err(DeliveryError{
                    kind: Kind::ExpectedJsonString,
                    detail: Some("Build cookbook 'path' value must be a string".to_string())
                })
            };
            let result = try!(Command::new("knife")
                 .arg("cookbook")
                 .arg("site")
                 .arg("download")
                 .arg(&name)
                 .arg("-f")
                 .arg(self.chef.join("build_cookbook.tgz").to_str().unwrap())
                 .current_dir(&self.root)
                 .output());
            if ! result.status.success() {
                let output = String::from_utf8_lossy(&result.stdout);
                let error = String::from_utf8_lossy(&result.stderr);
                return Err(DeliveryError{kind: Kind::SupermarketFailed, detail: Some(format!("Failed 'knife cookbook site download'\nOUT: {}\nERR: {}", &output, &error).to_string())});
            }
            let tar_result = try!(Command::new("tar")
                 .arg("zxf")
                 .arg(self.chef.join("build_cookbook.tgz").to_str().unwrap())
                 .current_dir(&self.chef)
                 .output());
            if ! tar_result.status.success() {
                let output = String::from_utf8_lossy(&tar_result.stdout);
                let error = String::from_utf8_lossy(&tar_result.stderr);
                return Err(DeliveryError{kind: Kind::TarFailed, detail: Some(format!("Failed 'tar zxf'\nOUT: {}\nERR: {}", &output, &error).to_string())});
            }
            let mv_result = try!(Command::new("mv")
                 .arg(self.chef.join(name).to_str().unwrap())
                 .arg(self.chef.join("build_cookbook").to_str().unwrap())
                 .current_dir(&self.chef)
                 .output());
            if ! mv_result.status.success() {
                let output = String::from_utf8_lossy(&mv_result.stdout);
                let error = String::from_utf8_lossy(&mv_result.stderr);
                return Err(DeliveryError{kind: Kind::MoveFailed, detail: Some(format!("Failed 'mv'\nOUT: {}\nERR: {}", &output, &error).to_string())});
            }
        } else {
            return Err(DeliveryError{ kind: Kind::MissingBuildCookbookName, detail: None })
        }
        Ok(())
    }

    fn setup_build_cookbook_from_chef_server(&self, name: &str) -> Result<(), DeliveryError> {
        try!(utils::mkdir_recursive(&self.chef.join("tmp_cookbook")));
        let result = try!(Command::new("knife")
                          .arg("download")
                          .arg(&format!("/cookbooks/{}", &name))
                          .arg("--chef-repo-path")
                          .arg(self.chef.join("tmp_cookbook").to_str().unwrap())
                          .current_dir(&self.root)
                          .output());
        if ! result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            let error = String::from_utf8_lossy(&result.stderr);
            return Err(DeliveryError{kind: Kind::ChefServerFailed, detail: Some(format!("Failed 'knife cookbook download'\nOUT: {}\nERR: {}", &output, &error).to_string())});
        }
        let mv_result = try!(Command::new("mv")
                             .arg(self.chef.join_many(&["tmp_cookbook", "cookbooks", &name]).to_str().unwrap())
                             .arg(self.chef.join("build_cookbook").to_str().unwrap())
                             .current_dir(&self.chef)
                             .output());
        if ! mv_result.status.success() {
            let output = String::from_utf8_lossy(&mv_result.stdout);
            let error = String::from_utf8_lossy(&mv_result.stderr);
            return Err(DeliveryError{kind: Kind::MoveFailed, detail: Some(format!("Failed 'mv'\nOUT: {}\nERR: {}", &output, &error).to_string())});
        }
        Ok(())
    }

    fn setup_build_cookbook_from_delivery(&self, build_cookbook: &Json, user: &str, server: &str) -> Result<(), DeliveryError> {
        let is_name = try!(build_cookbook.find("name").ok_or(DeliveryError{ kind: Kind::MissingBuildCookbookField, detail: Some("Missing name".to_string())}));
        let name = try!(is_name.as_string().ok_or(DeliveryError{
            kind: Kind::ExpectedJsonString,
            detail: Some("Build cookbook 'name' value must be a string".to_string())
        }));
        let is_ent = try!(build_cookbook.find("enterprise").ok_or(DeliveryError{ kind: Kind::MissingBuildCookbookField, detail: Some("Missing enterprise".to_string())}));
        let ent = try!(is_ent.as_string().ok_or(DeliveryError{
            kind: Kind::ExpectedJsonString,
            detail: Some("Build cookbook 'enterprise' value must be a string".to_string())
        }));
        let is_org = try!(build_cookbook.find("organization").ok_or(DeliveryError{ kind: Kind::MissingBuildCookbookField, detail: Some("Missing organization".to_string())}));
        let org = try!(is_org.as_string().ok_or(DeliveryError{
            kind: Kind::ExpectedJsonString,
            detail: Some("Build cookbook 'organization' value must be a string".to_string())
        }));
        let url = git::delivery_ssh_url(user, server, &ent, &org, &name);
        try!(git::git_command(&["clone", &url, self.chef.join("build_cookbook").to_str().unwrap()], &self.chef));
        Ok(())
    }

    fn setup_build_cookbook(&self, config: &Json, user: &str, server: &str) -> Result<(), DeliveryError> {
        let build_cookbook = try!(config.find("build_cookbook").ok_or(DeliveryError{
            kind: Kind::NoBuildCookbook,
            detail: None
        }));
        if build_cookbook.is_string() {
            let path = build_cookbook.as_string().unwrap();
            if path.contains("/") {
                return self.setup_build_cookbook_from_path(&self.repo.join(&path));
            } else {
                return self.setup_build_cookbook_from_chef_server(&path);
            }
        }
        let valid_paths = vec!["path", "git", "supermarket", "enterprise"];
        for path in valid_paths {
            let is_path = build_cookbook.find(path);
            if is_path.is_some() {
                match is_path.unwrap().as_string() {
                    Some(p) => {
                        match path {
                            "path" => return self.setup_build_cookbook_from_path(&self.repo.join(p)),
                            "git"  => return self.setup_build_cookbook_from_git(&build_cookbook, &p),
                            "supermarket" => return self.setup_build_cookbook_from_supermarket(&build_cookbook),
                            "enterprise" => return self.setup_build_cookbook_from_delivery(&build_cookbook, user, server),
                            "server" => {
                                let is_name = try!(build_cookbook.find("name")
                                                   .ok_or(DeliveryError{
                                                       kind: Kind::MissingBuildCookbookName,
                                                       detail: None
                                                   }));
                                let name = try!(is_name.as_string()
                                                .ok_or(DeliveryError{
                                                    kind: Kind::ExpectedJsonString,
                                                    detail: Some("Expected 'name' to be a string".to_string())
                                                }));
                                return self.setup_build_cookbook_from_chef_server(&name);
                            },
                            _ => unreachable!()
                        }
                    },
                    None => return Err(DeliveryError{
                        kind: Kind::ExpectedJsonString,
                        detail: Some(format!("Build cookbook '{}' value must be a string", path).to_string())
                    })
                }
            }
        }
        Err(DeliveryError{ kind: Kind::NoValidBuildCookbook, detail: None })
    }

    fn berks_vendor(&self, config: &Json) -> Result<(), DeliveryError> {
        try!(utils::remove_recursive(&self.chef.join("cookbooks")));
        if self.chef.join_many(&["build_cookbook", "Berksfile"]).is_file() {
            let mut command = Command::new("berks");
            command.arg("vendor");
            command.arg(&self.chef.join("cookbooks"));
            command.current_dir(&self.chef.join("build_cookbook"));
            let output = match command.output() {
                Ok(o) => o,
                Err(e) => { return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute berks vendor: {}", error::Error::description(&e)))}) },
            };
            if !output.status.success() {
                return Err(DeliveryError{ kind: Kind::BerksFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))});
            }
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            debug!("berks vendor stdout: {}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            debug!("berks vendor stderr: {}", stderr);
        } else {
            debug!("No Berksfile found; simply moving the cookbook");
            try!(utils::mkdir_recursive(&self.chef.join("cookbooks")));
            let bc_name = try!(self.build_cookbook_name(&config));
            let mv_result = try!(Command::new("mv")
                                 .arg(self.chef.join("build_cookbook").to_str().unwrap())
                                 .arg(self.chef.join_many(&["cookbooks", &bc_name]).to_str().unwrap())
                                 .current_dir(&self.chef)
                                 .output());
            if ! mv_result.status.success() {
                let output = String::from_utf8_lossy(&mv_result.stdout);
                let error = String::from_utf8_lossy(&mv_result.stderr);
                return Err(DeliveryError{kind: Kind::MoveFailed, detail: Some(format!("Failed 'mv'\nOUT: {}\nERR: {}", &output, &error).to_string())});
            }
        }
        Ok(())
    }

    /// This sets permissions in the workspace repo and cache directories. Going to
    /// want a windows implementation here.
    pub fn set_drop_permissions(&self) -> Result<(), DeliveryError> {
        let result = Command::new("chown")
            .arg("-R")
            .arg("dbuild:dbuild")
            .arg(self.repo.to_str().unwrap())
            .arg(self.chef.join("cookbooks").to_str().unwrap())
            .arg(self.chef.join("nodes").to_str().unwrap())
            .arg(self.cache.to_str().unwrap())
            .output();
       let output = match result {
            Ok(o) => o,
            Err(e) => { return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute chown: {}", error::Error::description(&e)))}) },
        };
        if !output.status.success() {
            return Err(DeliveryError{ kind: Kind::ChownFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))});
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        debug!("chmod stdout: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        debug!("chmod stderr: {}", stderr);
        Ok(())
    }

    pub fn build_cookbook_name(&self, config: &Json) -> Result<String, DeliveryError> {
        let bc_name = match config.find("build_cookbook") {
            Some(bc) => {
                if bc.is_string() {
                    let bc_string = bc.as_string().unwrap();
                    if bc_string.contains("/") {
                        let r = regex!(r"(.+)/(.+)");
                        let caps_result = r.captures(bc_string);
                        let caps = caps_result.unwrap();
                        caps.at(2).unwrap()
                    } else {
                        bc_string
                    }
                } else {
                    let is_bc_name = try!(bc.find("name").ok_or(DeliveryError{
                        kind: Kind::MissingBuildCookbookName,
                        detail: None}));
                    try!(is_bc_name.as_string().ok_or(DeliveryError{
                        kind: Kind::ExpectedJsonString,
                        detail: None}))
                }
            },
            None => return Err(DeliveryError{kind: Kind::NoValidBuildCookbook, detail: None})
        };
        Ok(bc_name.to_string())
    }

    pub fn run_job(&self, phase: &str, drop_privilege: Privilege) -> Result<(), DeliveryError> {
        let config = try!(job::config::load_config(&self.repo.join_many(&[".delivery", "config.json"])));
        let bc_name = try!(self.build_cookbook_name(&config));
        let mut command = Command::new("chef-client");
        command.arg("-z");
        command.arg("--force-formatter");
        match drop_privilege {
            Privilege::Drop => {
                try!(self.set_drop_permissions());
                command.arg("--user");
                command.arg("dbuild");
                command.arg("--group");
                command.arg("dbuild");
            },
            _ => {}
        }
        command.arg("-j");
        command.arg(&self.chef.join("dna.json").to_str().unwrap());
        command.arg("-c");
        command.arg(&self.chef.join("config.rb").to_str().unwrap());
        command.arg("-r");
        command.arg(&format!("{}::{}", bc_name, phase));
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.env("HOME", &self.cache.to_str().unwrap());
        match phase {
            "default" => command.env("DELIVERY_BUILD_SETUP", "TRUE"),
            _ => command.env("DELIVERY_BUILD_SETUP", "FALSE")
        };
        command.current_dir(&self.repo);
        let output = match command.output() {
            Ok(o) => o,
            Err(e) => { return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute chef-client: {}", error::Error::description(&e)))}) },
        };
        if !output.status.success() {
            return Err(DeliveryError{ kind: Kind::ChefFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))});
        }
        Ok(())
    }

    pub fn setup_chef_for_job(&self, user: &str, server: &str, change: Change) -> Result<(), DeliveryError> {
        let mut config_rb = try!(File::create(&self.chef.join("config.rb")));
        try!(config_rb.write_all(b"file_cache_path File.expand_path(File.join(File.dirname(__FILE__), '..', 'cache'))
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
"));
        try!(utils::chmod(&self.chef.join("config.rb"), "0644"));
        let config = try!(job::config::load_config(&self.repo.join_many(&[".delivery", "config.json"])));
        try!(self.setup_build_cookbook(&config, user, server));
        try!(self.berks_vendor(&config));
        let workspace_data = WorkspaceCompat{
            root: self.root.to_str().unwrap().to_string(),
            chef: self.chef.to_str().unwrap().to_string(),
            cache: self.cache.to_str().unwrap().to_string(),
            repo: self.repo.to_str().unwrap().to_string(),
        };
        let top = Top{
            workspace: workspace_data,
            change: change,
            config: config
        };
        let compat = BuilderCompat{
            workspace: self.root.to_str().unwrap().to_string(),
            repo: self.repo.to_str().unwrap().to_string(),
            cache: self.cache.to_str().unwrap().to_string(),
            build_id: "deprecated".to_string(),
            build_user: String::from_str("dbuild")
        };
        let dna = DNA{
            delivery: top,
            delivery_builder: compat
        };
        let mut dna_json = try!(File::create(&self.chef.join("dna.json")));
        let data = try!(json::encode(&dna));
        try!(dna_json.write_all(data.as_bytes()));
        try!(utils::chmod(&self.chef.join("dna.json"), "0644"));
        Ok(())
    }

    pub fn setup_repo_for_change(&self, git_url: &str, change_branch: &str, pipeline: &str, sha: &str) -> Result<(), DeliveryError> {
        if ! self.repo.join(".git").is_dir() {
            try!(git::git_command(&["clone", git_url, "."], &self.repo));
        }
        try!(git::git_command(&["remote", "update", "origin"], &self.repo));
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
    use std::path::PathBuf;

    #[test]
    fn new() {
        let root = PathBuf::from("clown");
        let w = Workspace::new(&root);
        assert_eq!(w.root, root);
        assert_eq!(w.chef, root.join("chef"));
        assert_eq!(w.cache, root.join("cache"));
        assert_eq!(w.repo, root.join("repo"));
    }
}
