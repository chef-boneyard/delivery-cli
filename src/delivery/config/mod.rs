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

pub use errors;
use errors::{DeliveryError, Kind};
use std::fs::File;
use std::default::Default;
use utils::say::{say, sayln};
use rustc_serialize::Encodable;
use std::path::PathBuf;
use toml;
use utils::mkdir_recursive;
use std::io::prelude::*;
use utils::path_join_many::PathJoinMany;

#[derive(RustcEncodable, Clone)]
pub struct Config {
    pub server: Option<String>,
    pub user: Option<String>,
    pub enterprise: Option<String>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub git_port: Option<String>,
    pub pipeline: Option<String>
}

impl Default for Config {
    fn default() -> Config {
        Config{
            server: None,
            enterprise: None,
            organization: None,
            project: None,
            user: None,
            git_port: Some(String::from_str("8989")),
            pipeline: Some(String::from_str("master"))
        }
    }
}

macro_rules! config_accessor_for {
    ($name:ident, $set_name:ident, $err_msg:expr) => (
        impl Config {
            pub fn $name(self) -> Result<String, DeliveryError> {
                match self.$name {
                    Some(v) => Ok(v.clone()),
                    None => Err(DeliveryError{ kind: Kind::MissingConfig, detail: Some(format!("{} or set it in your .toml config file", $err_msg))})
                }
            }

            pub fn $set_name(mut self, $name: &str) -> Config {
                if !$name.is_empty() {
                    self.$name = Some(String::from_str($name));
                }
                self
            }
        }
    )
}

config_accessor_for!(server, set_server, "Server not set; try --server");
config_accessor_for!(user, set_user, "User not set; try --user");
config_accessor_for!(enterprise, set_enterprise, "Enterprise not set; try --ent");
config_accessor_for!(organization, set_organization, "Organization not set; try --org");
config_accessor_for!(project, set_project, "Project not set; try --project");
config_accessor_for!(git_port, set_git_port, "Git Port not set");
config_accessor_for!(pipeline, set_pipeline, "Pipeline not set; try --for");

impl Config {
    pub fn load_config(cwd: &PathBuf) -> Result<Config, DeliveryError> {
        let have_config = Config::have_dot_delivery_cli(cwd);
        match have_config.as_ref() {
            Some(path) => {
                let toml = try!(Config::read_file(path));
                match Config::parse_config(&toml) {
                    Ok(c) => return Ok(c),
                    Err(_) => return Ok(Default::default())
                }
            },
            None => return Ok(Default::default())
        }
    }

    pub fn write_file(&self, path: &PathBuf) -> Result<(), DeliveryError> {
        let write_dir = path.join_many(&[".delivery"]);
        if !write_dir.is_dir() {
            try!(mkdir_recursive(&write_dir));
        }
        let write_path = path.join_many(&[".delivery", "cli.toml"]);
        say("white", "Writing configuration to ");
        sayln("yellow", &format!("{}", write_path.display()));
        let mut f = try!(File::create(&write_path));
        let toml_string = toml::encode_str(self);
        sayln("magenta", "New configuration");
        sayln("magenta", "-----------------");
        say("white", &toml_string);
        try!(f.write_all(toml_string.as_bytes()));
        Ok(())
    }

    pub fn parse_config(toml: &str) -> Result<Config, DeliveryError> {
        let mut parser = toml::Parser::new(toml);
        match parser.parse() {
            Some(value) => { return Config::set_values_from_toml_table(value); },
            None => {
                return Err(DeliveryError{
                    kind: Kind::ConfigParse,
                    detail: Some(format!("Parse errors: {:?}", parser.errors))
                });
            }
        }
    }

    fn set_values_from_toml_table(table: toml::Table) -> Result<Config, DeliveryError> {
        let mut config: Config = Default::default();
        config.server = Config::stringify_values(table.get("server"));
        config.project = Config::stringify_values(table.get("project"));
        config.enterprise = Config::stringify_values(table.get("enterprise"));
        config.organization = Config::stringify_values(table.get("organization"));
        config.user = Config::stringify_values(table.get("user"));
        config.git_port = Config::stringify_values(table.get("git_port"));
        return Ok(config);
    }

    fn read_file(path: &PathBuf) -> Result<String, DeliveryError>  {
        let mut toml_file = try!(File::open(path));
        let mut toml = String::new();
        try!(toml_file.read_to_string(&mut toml));
        Ok(toml)
    }

    fn stringify_values(toml_value: Option<&toml::Value>) -> Option<String> {
        match toml_value {
            Some(value) => {
                let is_string = value.as_str();
                match is_string {
                    Some(vstr) => return Some(String::from_str(vstr)),
                    None => return None
                }
            },
            None => {
                return None;
            }
        }
    }

    fn check_dot_delivery_cli(path: PathBuf) -> Option<PathBuf> {
        let dot_git = path.join_many(&[".delivery", "cli.toml"]);
        debug!("Checking {}", dot_git.display());
        let is_file: Option<PathBuf> = if dot_git.is_file() {
            Some(dot_git)
        } else {
            None
        };
        is_file
    }

    fn have_dot_delivery_cli(orig_path: &PathBuf) -> Option<PathBuf> {
        let mut path = orig_path.clone();
        loop {
            let check_result: Option<PathBuf> = Config::check_dot_delivery_cli(path.clone());
            match check_result.as_ref() {
                Some(_) => { return check_result.clone() }
                None => {
                    if path.pop() { } else { return check_result.clone() }
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn parse_config() {
        let toml = r#"
            server = "127.0.0.1"
            enterprise = "chef"
            organization = "chef"
            user = "adam"
"#;
        let config_result = Config::parse_config(toml);
        match config_result {
            Ok(config) => {
                assert_eq!(config.server, Some(String::from_str("127.0.0.1")));
                assert_eq!(config.git_port, None);
            },
            Err(e) => {
                panic!("Failed to parse: {:?}", e.detail)
            }
        }
    }
}
