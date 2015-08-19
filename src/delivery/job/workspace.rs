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
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use utils;
use utils::path_join_many::PathJoinMany;
use std::error;
use config::Config;

#[derive(RustcDecodable, Debug)]
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

// We want this to encode as strings, not as vectors of bytes. It's
// cool - I accept we'll panic if its not a utf8 string.
impl Encodable for Workspace {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("Workspace", 0, |encoder| {
            try!(encode_path_field(encoder, 0usize, "root",  &self.root));
            try!(encode_path_field(encoder, 1usize, "chef",  &self.chef));
            try!(encode_path_field(encoder, 2usize, "cache", &self.cache));
            try!(encode_path_field(encoder, 3usize, "repo",  &self.repo));
            try!(encode_path_field(encoder, 4usize, "ssh_wrapper",
                                   &self.ssh_wrapper));
            Ok(())
        })
    }
}

fn encode_path_field<S: Encoder>(encoder: &mut S, size: usize,
                                 name: &str, p: &Path) -> Result<(), S::Error> {
    encoder.emit_struct_field(name, size, |e| encode_path(p, e))
}

fn encode_path<S: Encoder>(p: &Path, encoder: &mut S) -> Result<(), S::Error> {
    path_to_string(p).encode(encoder)
}

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

    pub fn build(&self) -> Result<(), DeliveryError> {
        try!(utils::mkdir_recursive(&self.root));
        // These two directories will get chown'd to the build user,
        // so we want to make sure they exist.
        try!(utils::mkdir_recursive(&self.chef.join("nodes")));
        try!(utils::mkdir_recursive(&self.chef.join("cookbooks")));
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
        let build_cookbook_path = &self.chef.join("build_cookbook");
        try!(git::git_command(&["clone", git_url,
                                &path_to_string(build_cookbook_path)],
                              &self.chef));
        try!(git::git_command(&["checkout", &branch], build_cookbook_path));
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
            let result = try!(utils::make_command("knife")
                 .arg("cookbook")
                 .arg("site")
                 .arg("download")
                 .arg(&name)
                 .arg("-f")
                 .arg(&path_to_string(&self.chef.join("build_cookbook.tgz")))
                 .current_dir(&self.root)
                 .output());
            if ! result.status.success() {
                let output = String::from_utf8_lossy(&result.stdout);
                let error = String::from_utf8_lossy(&result.stderr);
                return Err(DeliveryError{kind: Kind::SupermarketFailed, detail: Some(format!("Failed 'knife cookbook site download'\nOUT: {}\nERR: {}", &output, &error).to_string())});
            }
            let tar_result = try!(utils::make_command("tar")
                 .arg("zxf")
                 .arg(&path_to_string(&self.chef.join("build_cookbook.tgz")))
                 .current_dir(&self.chef)
                 .output());
            if ! tar_result.status.success() {
                let output = String::from_utf8_lossy(&tar_result.stdout);
                let error = String::from_utf8_lossy(&tar_result.stderr);
                return Err(DeliveryError{kind: Kind::TarFailed, detail: Some(format!("Failed 'tar zxf'\nOUT: {}\nERR: {}", &output, &error).to_string())});
            }
            let mv_result = try!(utils::make_command("mv")
                 .arg(&path_to_string(&self.chef.join(name)))
                 .arg(&path_to_string(&self.chef.join("build_cookbook")))
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
            return Err(DeliveryError{kind: Kind::ChefServerFailed, detail: Some(format!("Failed 'knife cookbook download'\nOUT: {}\nERR: {}", &output, &error).to_string())});
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
            return Err(DeliveryError{kind: Kind::MoveFailed, detail: Some(format!("Failed 'mv'\nOUT: {}\nERR: {}", &output, &error).to_string())});
        }
        Ok(())
    }

    fn setup_build_cookbook_from_delivery(&self, build_cookbook: &Json, toml_config: &Config) -> Result<(), DeliveryError> {
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

        let build_cookbook_config = toml_config.clone().set_enterprise(ent)
                                                       .set_organization(org)
                                                       .set_project(name);

        let url = try!(build_cookbook_config.delivery_git_ssh_url());
        try!(git::git_command(&["clone", &url, self.chef.join("build_cookbook").to_str().unwrap()], &self.chef));
        Ok(())
    }

    fn setup_build_cookbook(&self, toml_config: &Config, config: &Json) -> Result<(), DeliveryError> {
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
                            "enterprise" => return self.setup_build_cookbook_from_delivery(&build_cookbook, toml_config),
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
                                 .arg(&path_to_string(&self.chef.join("build_cookbook")))
                                 .arg(&path_to_string(&self.chef.join_many(&["cookbooks", &bc_name])))
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

    /// This sets permissions in the workspace repo and cache directories.
    pub fn set_drop_permissions(&self) -> Result<(), DeliveryError> {
        let paths_to_chown = &[&self.repo,
                               &self.chef.join("cookbooks"),
                               &self.chef.join("nodes"),
                               &self.cache];
        utils::chown_all("dbuild:dbuild", paths_to_chown)
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

    pub fn run_job(&self, phase_arg: &str, drop_privilege: &Privilege) -> Result<(), DeliveryError> {
        let config = try!(job::config::load_config(&self.repo.join_many(&[".delivery", "config.json"])));
        let bc_name = try!(self.build_cookbook_name(&config));
        let run_list = {
            let phases: Vec<String> = phase_arg.split(" ")
                .map(|p| format!("{}::{}", bc_name, p)).collect();
            phases.join(",")
        };
        let mut command = utils::make_command("chef-client");
        command.arg("-z").arg("--force-formatter");
        try!(self.handle_privilege_drop(drop_privilege, &mut command));
        command.arg("-j")
            .arg(&path_to_string(&self.chef.join("dna.json")))
            .arg("-c")
            .arg(&path_to_string(&self.chef.join("config.rb")))
            .arg("-r")
            .arg(run_list)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("HOME", &path_to_string(&self.cache))
            .current_dir(&self.repo);
        match phase_arg {
            "default" => command.env("DELIVERY_BUILD_SETUP", "TRUE"),
            _ => command.env("DELIVERY_BUILD_SETUP", "FALSE")
        };
        let output = match command.output() {
            Ok(o) => o,
            Err(e) => { return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute chef-client: {}", error::Error::description(&e)))}) },
        };
        if !output.status.success() {
            return Err(DeliveryError{ kind: Kind::ChefFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))});
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
                              toml_config: &Config,
                              change: Change) -> Result<(), DeliveryError> {
        let config_rb_path = &self.chef.join("config.rb");
        let mut config_rb = try!(File::create(config_rb_path));
        try!(utils::chmod(config_rb_path, "0644"));
        try!(config_rb.write_all(CONFIG_RB.as_bytes()));
        let proj_config_path = &self.repo.join_many(&[".delivery",
                                                      "config.json"]);
        let config = try!(job::config::load_config(proj_config_path));
        try!(self.setup_build_cookbook(toml_config, &config));
        try!(self.berks_vendor(&config));
        let workspace_data = WorkspaceCompat{
            root: path_to_string(&self.root),
            chef: path_to_string(&self.chef),
            cache: path_to_string(&self.cache),
            repo: path_to_string(&self.repo),
            ssh_wrapper: path_to_string(&self.ssh_wrapper),
        };
        let top = Top{
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
        let dna_json_path = &self.chef.join("dna.json");
        let mut dna_json = try!(File::create(dna_json_path));
        try!(utils::chmod(dna_json_path, "0644"));
        let data = try!(json::encode(&dna));
        try!(dna_json.write_all(data.as_bytes()));
        Ok(())
    }

    pub fn setup_repo_for_change(&self, git_url: &str, change_branch: &str, pipeline: &str, sha: &str) -> Result<(), DeliveryError> {
        if ! self.repo.join(".git").is_dir() {
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

// Convert a path into a String. Panic if the path contains
// non-unicode sequences.
fn path_to_string(p: &Path) -> String {
    match p.to_str() {
        Some(s) => s.to_string(),
        None => {
            let s = format!("invalid path (non-unicode): {}",
                            p.to_string_lossy());
            panic!(s)
        }
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
