#![allow(unstable)]
use std::old_io::{self, File};
use std::old_io::fs;
use errors::{DeliveryError, Kind};
use std::old_io::fs::PathExtensions;
use git;
use rustc_serialize::{Encodable, Encoder};
use rustc_serialize::json::{self, Json};
use job::dna::{Top, DNA};
use job::change::Change;
use uuid::Uuid;
use job;
use std::old_io::process::Command;

#[derive(RustcDecodable)]
pub struct Workspace {
    pub root: Path,
    pub chef: Path,
    pub cache: Path,
    pub repo: Path
}

// We want this to encode as strings, not as vectors of bytes.
// It's cool - I accept we'll be lossy if its not a utf8 string.
impl Encodable for Workspace {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("Workspace", 0, |encoder| {
            try!(encoder.emit_struct_field( "root", 0us, |encoder| self.root.as_str().unwrap().encode(encoder)));
            try!(encoder.emit_struct_field( "chef", 1us, |encoder| self.chef.as_str().unwrap().encode(encoder)));
            try!(encoder.emit_struct_field( "cache", 2us, |encoder| self.cache.as_str().unwrap().encode(encoder)));
            try!(encoder.emit_struct_field( "repo", 3us, |encoder| self.repo.as_str().unwrap().encode(encoder)));
            Ok(())
        })
    }
}

impl Workspace {
    pub fn new(root: &Path) -> Workspace {
        Workspace{
            root: root.clone(),
            chef: root.join("chef"),
            cache: root.join("cache"),
            repo: root.join("repo")
        }
    }

    pub fn build(&self) -> Result<(), DeliveryError> {
        try!(fs::mkdir_recursive(&self.root, old_io::USER_RWX));
        try!(fs::mkdir_recursive(&self.chef, old_io::USER_RWX));
        try!(fs::mkdir_recursive(&self.cache, old_io::USER_RWX));
        try!(fs::mkdir_recursive(&self.repo, old_io::USER_RWX));
        Ok(())
    }

    fn reset_repo(&self, git_ref: &str) -> Result<(), DeliveryError> {
        try!(git::git_command(&["reset", "--hard", git_ref], &self.repo));
        try!(git::git_command(&["clean", "-x", "-f", "-d", "-q"], &self.repo));
        Ok(())
    }

    fn berks_vendor(&self, config: &Json) -> Result<(), DeliveryError> {
        let build_cookbook = match config.find("build_cookbook") {
            Some(ref bc) => bc.as_string().unwrap(),
            None => return Err(DeliveryError{kind: Kind::NoBuildCookbook, detail: None})
        };
        let mut command = Command::new("berks");
        command.arg("vendor");
        command.arg(&self.chef.join("cookbooks"));
        command.cwd(&self.repo.join(build_cookbook.as_slice()));
        let output = match command.output() {
            Ok(o) => o,
            Err(e) => { return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute berks vendor: {}", e.desc))}) },
        };
        if !output.status.success() {
            return Err(DeliveryError{ kind: Kind::BerksFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(output.output.as_slice()), String::from_utf8_lossy(output.error.as_slice())))});
        }
        let stdout = String::from_utf8_lossy(output.output.as_slice()).to_string();
        debug!("berks vendor stdout: {}", stdout);
        let stderr = String::from_utf8_lossy(output.error.as_slice()).to_string();
        debug!("berks vendor stderr: {}", stderr);
        Ok(())
    }

    pub fn run_job(&self, phase: &str) -> Result<(), DeliveryError> {
        let config = try!(job::config::load_config(&self.repo.join_many(&[".delivery", "config.json"])));
        let build_cookbook = match config.find("build_cookbook") {
            Some(ref bc) => bc.as_string().unwrap(),
            None => return Err(DeliveryError{kind: Kind::NoBuildCookbook, detail: None})
        };
        let bc_path = Path::new(build_cookbook);
        let bc_name = String::from_utf8_lossy(bc_path.filename().unwrap());
        let mut command = Command::new("chef-client");
        command.arg("-z");
        command.arg("-j");
        command.arg(self.chef.join("dna.json").as_str().unwrap().as_slice());
        command.arg("-c");
        command.arg(self.chef.join("config.rb").as_str().unwrap().as_slice());
        command.arg("-r");
        command.arg(format!("{}::{}", bc_name, phase).as_slice());
        command.stdout(old_io::process::StdioContainer::InheritFd(1));
        command.stderr(old_io::process::StdioContainer::InheritFd(2));
        command.cwd(&self.repo);
        let output = match command.output() {
            Ok(o) => o,
            Err(e) => { return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute chef-client: {}", e.desc))}) },
        };
        if !output.status.success() {
            return Err(DeliveryError{ kind: Kind::GitFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(output.output.as_slice()), String::from_utf8_lossy(output.error.as_slice())))});
        }
        Ok(())
    }

    pub fn setup_chef_for_job(&self) -> Result<(), DeliveryError> {
        let mut config_rb = File::create(&self.chef.join("config.rb"));
        try!(config_rb.write(b"file_cache_path File.expand_path(File.join(File.dirname(__FILE__), '..', 'cache'))
cache_type 'BasicFile'
cache_options(:path => File.join(file_cache_path, 'checksums'))
cookbook_path File.expand_path(File.join(File.dirname(__FILE__), 'cookbooks'))
file_backup_path File.expand_path(File.join(File.dirname(__FILE__), 'cache', 'job-backup'))
"));
        let change = Change{
            enterprise: String::from_str("ent"),
            organization: String::from_str("org"),
            project: String::from_str("project"),
            pipeline: String::from_str("project"),
            change_id: Uuid::new_v4(),
            patchset_number: 1f64,
            stage: String::from_str("verify"),
            stage_run_id: 1f64,
            phase: String::from_str("unit"),
            phase_run_id: 1f64,
            git_url: String::from_str("git_url"),
            sha: String::from_str("sha"),
            patchset_branch: String::from_str("master"),
            delivery_api_url: Some(String::from_str("")),
            delivery_data_url: Some(String::from_str("")),
            delivery_change_url: Some(String::from_str("")),
            log_level: String::from_str("info"),
            token: Some(String::from_str(""))
        };
        let config = try!(job::config::load_config(&self.repo.join_many(&[".delivery", "config.json"])));
        try!(self.berks_vendor(&config));
        let top = Top{
            workspace: Workspace::new(&self.root),
            change: change,
            config: config
        };
        let dna = DNA{
            delivery: top
        };
        let mut dna_json = File::create(&self.chef.join("dna.json"));
        let data = try!(json::encode(&dna));
        try!(dna_json.write(data.as_bytes()));
        Ok(())
    }

    pub fn setup_repo_for_change(&self, git_url: &str, change_branch: &str, pipeline: &str, sha: &str) -> Result<(), DeliveryError> {
        if ! self.repo.join(".git").is_dir() {
            try!(git::git_command(&["clone", git_url, "."], &self.repo));
        }
        try!(git::git_command(&["remote", "update"], &self.repo));
        try!(self.reset_repo("HEAD"));
        try!(git::git_command(&["checkout", pipeline], &self.repo));
        try!(self.reset_repo(format!("remotes/origin/{}", pipeline).as_slice()));
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

    #[test]
    fn new() {
        let root = Path::new("clown");
        let w = Workspace::new(&root);
        assert_eq!(w.root, root);
        assert_eq!(w.chef, root.join("chef"));
        assert_eq!(w.cache, root.join("cache"));
        assert_eq!(w.repo, root.join("repo"));
    }
}
