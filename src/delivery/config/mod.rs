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
    pub api_port: Option<String>,
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
            api_port: None,
            enterprise: None,
            organization: None,
            project: None,
            user: None,
            git_port: Some(String::from("8989")),
            pipeline: Some(String::from("master"))
        }
    }
}

macro_rules! config_accessor_for {
    ($name:ident, $set_name:ident, $err_msg:expr) => (
        impl Config {
            pub fn $name(&self) -> Result<String, DeliveryError> {
                match self.$name {
                    Some(ref v) => Ok(v.clone()),
                    None => Err(DeliveryError{ kind: Kind::MissingConfig, detail: Some(String::from($err_msg)) })
                }
            }

            pub fn $set_name(mut self, $name: &str) -> Config {
                if !$name.is_empty() {
                    self.$name = Some(String::from($name));
                }
                self
            }
        }
    )
}

config_accessor_for!(server, set_server, "Server not set; try --server or set it in your .toml config file");
config_accessor_for!(api_port, set_api_port, "API port not set; try --api-port or set it in your .toml config file");
config_accessor_for!(user, set_user, "User not set; try --user or set it in your .toml config file");
config_accessor_for!(enterprise, set_enterprise, "Enterprise not set; try --ent or set it in your .toml config file");
config_accessor_for!(organization, set_organization, "Organization not set; try --org or set it in your .toml config file");
config_accessor_for!(project, set_project, "Project not set; try --project or set it in your .toml config file");
config_accessor_for!(git_port, set_git_port, "Git Port not set; please set it in your .toml config file");
config_accessor_for!(pipeline, set_pipeline, "Pipeline not set; try --for or set it in your .toml config file");

impl Config {

    /// Return the host and port at which we can access the Delivery
    /// API. By default, we assume the use of HTTPS on the standard
    /// port `443`. Unless a port is specified in the configuration,
    /// we'll just return the server name; otherwise we append the
    /// port.
    pub fn api_host_and_port(&self) -> Result<String, DeliveryError> {
        let s = try!(self.server());
        return Ok(match self.api_port {
            Some(ref p) => format!("{}:{}", s, p),
            None    => s
        });
    }

    /// Returns the SSH URL to talk to Delivery's Git
    pub fn delivery_git_ssh_url(&self) -> Result<String, DeliveryError> {
        let s = try!(self.server());
        let host_and_port = match self.git_port {
            Some(ref p) => format!("{}:{}", s, p),
            None    => s // TODO: Currently we *always* have a git port
        };
        let u = try!(self.user());
        let e = try!(self.enterprise());
        let o = try!(self.organization());
        let p = try!(self.project());
        Ok(format!("ssh://{}@{}@{}/{}/{}/{}", u, e, host_and_port, e, o, p))
    }

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
        config.server = stringify_or("server", &table, config.server);
        config.api_port = stringify_or("api_port", &table, config.api_port);
        config.pipeline = stringify_or("pipeline", &table, config.pipeline);
        config.project = stringify_or("project", &table, config.project);
        config.enterprise = stringify_or("enterprise", &table,
                                         config.enterprise);
        config.organization = stringify_or("organization", &table,
                                           config.organization);
        config.user = stringify_or("user", &table, config.user);
        config.git_port = stringify_or("git_port", &table, config.git_port);
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
                    Some(vstr) => return Some(String::from(vstr)),
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

fn stringify_or(key: &str, table: &toml::Table, default: Option<String>) -> Option<String> {
    Config::stringify_values(table.get(key)).or(default)
}


#[cfg(test)]
mod tests {
    use super::Config;
    use std::default::Default;

    #[test]
    fn parse_config_with_defaults() {
        let toml = r#"
            server = "127.0.0.1"
            enterprise = "chef"
            user = "adam"
"#;
        let config_result = Config::parse_config(toml);
        match config_result {
            Ok(config) => {
                assert_eq!(Some(String::from("127.0.0.1")), config.server);
                assert_eq!(Some("8989".to_string()), config.git_port);
                assert_eq!(Some("master".to_string()), config.pipeline);
                assert_eq!(None, config.organization);
            },
            Err(e) => {
                panic!("Failed to parse: {:?}", e.detail)
            }
        }
    }

    #[test]
    fn parse_config_override_defaults() {
        let toml = r#"
            server = "127.0.0.1"
            enterprise = "chef"
            user = "adam"
            git_port = "4151"
            pipeline = "dev"
"#;
        let config_result = Config::parse_config(toml);
        match config_result {
            Ok(config) => {
                assert_eq!(Some("4151".to_string()), config.git_port);
                assert_eq!(Some("dev".to_string()), config.pipeline);
                assert_eq!(Some(String::from("127.0.0.1")), config.server);
                assert_eq!(None, config.organization);
            },
            Err(e) => {
                panic!("Failed to parse: {:?}", e.detail)
            }
        }
    }

    #[test]
    fn test_api_url_with_port() {
        let mut conf  = Config::default();
        conf.server   = Some("127.0.0.1".to_string());
        conf.api_port = Some("2112".to_string());
        assert_eq!("127.0.0.1:2112".to_string(),
                   conf.api_host_and_port().unwrap());
    }

    #[test]
    fn test_api_url_without_port() {
        let mut conf = Config::default();
        conf.server  = Some("127.0.0.1".to_string());
        assert!(conf.api_port.is_none());
        assert_eq!("127.0.0.1".to_string(),
                   conf.api_host_and_port().unwrap());
    }

    #[test]
    fn test_api_url_without_server() {
        let conf = Config::default();
        assert!(conf.server.is_none());
        assert!(conf.api_host_and_port().is_err());
    }

    #[test]
    fn test_git_url_with_default_port() {
        let mut conf      = Config::default();
        conf.server       = Some("127.0.0.1".to_string());
        conf.user         = Some("user".to_string());
        conf.enterprise   = Some("ent".to_string());
        conf.organization = Some("org".to_string());
        conf.project      = Some("proj".to_string());
        assert_eq!("ssh://user@ent@127.0.0.1:8989/ent/org/proj".to_string(),
                   conf.delivery_git_ssh_url().unwrap());
    }

    #[test]
    fn test_git_url_with_port() {
        let mut conf      = Config::default();
        conf.server       = Some("127.0.0.1".to_string());
        conf.user         = Some("user".to_string());
        conf.enterprise   = Some("ent".to_string());
        conf.organization = Some("org".to_string());
        conf.project      = Some("proj".to_string());
        conf.git_port     = Some("2112".to_string());
        assert_eq!("ssh://user@ent@127.0.0.1:2112/ent/org/proj".to_string(),
                   conf.delivery_git_ssh_url().unwrap());
    }

    #[test]
    fn test_git_url_without_server() {
        let conf = Config::default();
        assert!(conf.server.is_none());
        assert!(conf.delivery_git_ssh_url().is_err());
    }
}
