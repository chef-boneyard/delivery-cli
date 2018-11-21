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
use std::clone::Clone;
use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use toml;
use types::DeliveryResult;
use utils::path_ext::{is_dir, is_file};
use utils::path_join_many::PathJoinMany;
use utils::{mkdir_recursive, read_file};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub server: Option<String>,
    pub api_port: Option<String>,
    pub api_protocol: Option<String>,
    pub user: Option<String>,
    pub enterprise: Option<String>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub git_port: Option<String>,
    pub pipeline: Option<String>,
    pub token_file: Option<String>,
    pub generator: Option<String>,
    pub non_interactive: Option<bool>,
    pub auto_bump: Option<bool>,
    pub config_json: Option<String>,
    pub saml: Option<bool>,
    pub fips: Option<bool>,
    pub fips_git_port: Option<String>,
    pub fips_custom_cert_filename: Option<String>,
    pub a2_mode: Option<bool>,
}

pub mod url_format;

impl Default for Config {
    fn default() -> Config {
        Config {
            server: None,
            api_port: None,
            api_protocol: Some(String::from("https")),
            enterprise: None,
            organization: None,
            project: None,
            user: None,
            git_port: Some(String::from("8989")),
            pipeline: Some(String::from("master")),
            token_file: None,
            generator: None,
            non_interactive: None,
            auto_bump: None,
            config_json: None,
            saml: None,
            fips: None,
            fips_git_port: None,
            fips_custom_cert_filename: None,
            a2_mode: None
        }
    }
}


macro_rules! config_accessor_for {
    ($name:ident, $set_name:ident, $err_msg:expr) => {
        impl Config {
            pub fn $name(&self) -> DeliveryResult<String> {
                match self.$name {
                    Some(ref v) => Ok(v.clone()),
                    None => Err(DeliveryError {
                        kind: Kind::MissingConfig,
                        detail: Some(String::from($err_msg)),
                    }),
                }
            }

            pub fn $set_name(mut self, $name: &str) -> Config {
                if !$name.is_empty() {
                    self.$name = Some(String::from($name));
                }
                self
            }
        }
    };
}

// TODO: DRY this up with above
macro_rules! config_bool_accessor_for {
    ($name:ident, $set_name:ident, $err_msg:expr) => {
        impl Config {
            pub fn $name(&self) -> DeliveryResult<bool> {
                match self.$name {
                    Some(ref v) => Ok(v.clone()),
                    None => Err(DeliveryError {
                        kind: Kind::MissingConfig,
                        detail: Some(String::from($err_msg)),
                    }),
                }
            }

            pub fn $set_name(mut self, $name: bool) -> Config {
                self.$name = Some($name);
                self
            }
        }
    };
}



config_accessor_for!(
    server,
    set_server,
    "Server not set; try --server or set it in your .toml config file"
);
config_accessor_for!(
    api_port,
    set_api_port,
    "API port not set; try --api-port or set it in your .toml config file"
);
config_accessor_for!(
    api_protocol,
    set_api_protocol,
    "api_protocol not set; set it in your cli.toml"
);
config_accessor_for!(
    user,
    set_user,
    "User not set; try --user or set it in your .toml config file"
);
config_accessor_for!(
    enterprise,
    set_enterprise,
    "Enterprise not set; try --ent or set it in your .toml config file"
);
config_accessor_for!(
    organization,
    set_organization,
    "Organization not set; try --org or set it in your .toml config file"
);
config_accessor_for!(
    project,
    set_project,
    "Project not set; try --project or set it in your .toml config file"
);
config_accessor_for!(
    git_port,
    set_git_port,
    "Git Port not set; please set it in your .toml config file"
);
config_accessor_for!(
    pipeline,
    set_pipeline,
    "Pipeline not set; try --for or set it in your .toml config file"
);
config_accessor_for!(
    token_file,
    set_token_file,
    "token_file not set; set it in your cli.toml"
);
config_accessor_for!(
    generator,
    set_generator,
    "build_cookbook generator not set; set it in your cli.toml"
);
config_accessor_for!(
    config_json,
    set_config_json,
    "config_json not set; set it in your cli.toml"
);
config_accessor_for!(
    fips_git_port,
    set_fips_git_port,
    "You did not set the fips_git_port. Set this value in your cli.toml or pass --fips-git-port.\nIt should be set to any port that is free and open on localhost (i.e. `fips_git_port = \"36534\"` in your cli.toml)."
);

config_bool_accessor_for!(
    a2_mode,
    set_a2_mode,
    "You did not set the a2_mode. Set this value in your cli.toml."
);


impl Config {
    /// Return the host and port at which we can access the Delivery
    /// API. By default, we assume the use of HTTPS on the standard
    /// port `443`. Unless a port is specified in the configuration,
    /// we'll just return the server name; otherwise we append the
    /// port.
    pub fn api_host_and_port(&self) -> DeliveryResult<String> {
        let s = try!(self.server());
        return Ok(match self.api_port {
            Some(ref p) => format!("{}:{}", s, p),
            None => s,
        });
    }

    /// Return the host and port, suffixed with the workflow resource if we are in A2 mode
    pub fn api_base_resource(&self) -> DeliveryResult<String> {
        let host_and_port = try!(self.api_host_and_port());
        let resource_base = if self.a2_mode.unwrap_or(false) {
            format!("{}/workflow", host_and_port)
        } else {
            host_and_port
        };
        return Ok(resource_base);
    }

    /// Returns the SSH URL to talk to Delivery's Git
    pub fn delivery_git_ssh_url(&self) -> DeliveryResult<String> {
        if self.fips.unwrap_or(false) {
            self.delivery_git_fips_enabled_url()
        } else {
            self.delivery_git_ssh_standard_url()
        }
    }

    fn delivery_git_ssh_standard_url(&self) -> DeliveryResult<String> {
        let s = try!(self.server());
        let host_and_port = match self.git_port {
            Some(ref p) => format!("{}:{}", s, p),
            None => s, // TODO: Currently we *always* have a git port
        };
        let u = try!(self.user());
        let e = try!(self.enterprise());
        let o = try!(self.organization());
        let p = try!(self.project());
        Ok(format!(
            "ssh://{}@{}@{}/{}/{}/{}",
            u, e, host_and_port, e, o, p
        ))
    }

    fn delivery_git_fips_enabled_url(&self) -> DeliveryResult<String> {
        let host_and_port = format!("{}:{}", "localhost", try!(self.fips_git_port()));
        let u = try!(self.user());
        let e = try!(self.enterprise());
        let o = try!(self.organization());
        let p = try!(self.project());
        Ok(format!(
            "ssh://{}@{}@{}/{}/{}/{}",
            u, e, host_and_port, e, o, p
        ))
    }

    pub fn load_config(cwd: &PathBuf) -> DeliveryResult<Self> {
        let have_config = Config::dot_delivery_cli_path(cwd);
        match have_config.as_ref() {
            Some(path) => {
                let toml = read_file(path)?;
                match Config::parse_config(&toml) {
                    Ok(c) => return Ok(c),
                    Err(_) => return Ok(Default::default()),
                }
            }
            None => return Ok(Default::default()),
        }
    }

    pub fn write_file<P>(&self, path: P) -> DeliveryResult<String>
    where
        P: AsRef<Path>,
    {
        let write_dir = path.as_ref().join_many(&[".delivery"]);
        if !is_dir(&write_dir) {
            try!(mkdir_recursive(&write_dir));
        }
        let write_file = write_dir.join_many(&["cli.toml"]);
        let mut f = try!(File::create(&write_file));
        let toml_string = toml::to_string(self)?;
        try!(f.write_all(toml_string.as_bytes()));
        Ok(toml_string)
    }

    pub fn parse_config(toml_str: &str) -> DeliveryResult<Self> {
        let mut config: Config = Default::default();
        let toml = toml::from_str::<Config>(toml_str)?;
        config.override_with(toml);
        Ok(config)
    }

    // Override Config
    //
    // This method will override the instance of the Config
    // `self` with another provided Config. This is useful,
    // for example, if you need to merge two configs.
    pub fn override_with(&mut self, config: Config) {
        // (afiune) TODO: I think we could do better by implementing some
        // sort of Iterator or other thing that let us loop through the
        // fields, but for now this is good enough.
        //
        // If the `config` has some new config, override `self`
        if config.server.is_some() {
            self.server = config.server
        }
        if config.api_port.is_some() {
            self.api_port = config.api_port
        }
        if config.pipeline.is_some() {
            self.pipeline = config.pipeline
        }
        if config.project.is_some() {
            self.project = config.project
        }
        if config.enterprise.is_some() {
            self.enterprise = config.enterprise
        }
        if config.organization.is_some() {
            self.organization = config.organization
        }
        if config.user.is_some() {
            self.user = config.user
        }
        if config.git_port.is_some() {
            self.git_port = config.git_port
        }
        if config.token_file.is_some() {
            self.token_file = config.token_file
        }
        if config.generator.is_some() {
            self.generator = config.generator
        }
        if config.non_interactive.is_some() {
            self.non_interactive = config.non_interactive
        }
        if config.auto_bump.is_some() {
            self.auto_bump = config.auto_bump
        }
        if config.config_json.is_some() {
            self.config_json = config.config_json
        }
        if config.saml.is_some() {
            self.saml = config.saml
        }
        if config.fips.is_some() {
            self.fips = config.fips
        }
        if config.fips_git_port.is_some() {
            self.fips_git_port = config.fips_git_port
        }
        if config.fips_custom_cert_filename.is_some() {
            self.fips_custom_cert_filename = config.fips_custom_cert_filename
        }
        if config.api_protocol.is_some() {
            self.api_protocol = config.api_protocol
        }
        if config.a2_mode.is_some() {
            self.a2_mode = config.a2_mode
        }
    }

    fn check_dot_delivery_cli(path: PathBuf) -> Option<PathBuf> {
        let dot_git = path.join_many(&[".delivery", "cli.toml"]);
        debug!("Checking {}", dot_git.display());
        let is_file: Option<PathBuf> = if is_file(&dot_git) {
            Some(dot_git)
        } else {
            None
        };
        is_file
    }

    pub fn dot_delivery_cli_path<P>(orig_path: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        let mut path = orig_path.as_ref().to_owned();
        loop {
            let check_result: Option<PathBuf> = Config::check_dot_delivery_cli(path.clone());
            match check_result.as_ref() {
                Some(_) => return check_result.clone(),
                None => {
                    if path.pop() {
                    } else {
                        return check_result.clone();
                    }
                }
            }
        }
    }
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
                assert_eq!(None, config.api_port);
                assert_eq!(Some("https".to_string()), config.api_protocol);
                assert_eq!(Some("8989".to_string()), config.git_port);
                assert_eq!(Some("master".to_string()), config.pipeline);
                assert_eq!(None, config.organization);
                assert_eq!(None, config.token_file);
                assert_eq!(None, config.generator);
                assert_eq!(None, config.non_interactive);
                assert_eq!(None, config.auto_bump);
                assert_eq!(None, config.config_json);
                assert_eq!(None, config.saml);
                assert_eq!(None, config.fips);
                assert_eq!(None, config.fips_git_port);
                assert_eq!(Some(false), config.a2_mode);
            }
            Err(e) => panic!("Failed to parse: {:?}", e.detail),
        }
    }

    #[test]
    fn parse_config_override_defaults() {
        let toml = r#"
            server = "127.0.0.1"
            enterprise = "chef"
            user = "adam"
            git_port = "4151"
            api_protocol = "http"
            api_port = "7643"
            pipeline = "dev"
            non_interactive = true
            auto_bump = true
            config_json = "/path/to/my/custom/config.json"
            saml = true
            fips = true
            fips_git_port = "55555"
            a2_mode = true
"#;
        let config_result = Config::parse_config(toml);
        match config_result {
            Ok(config) => {
                assert_eq!(Some("4151".to_string()), config.git_port);
                assert_eq!(Some("7643".to_string()), config.api_port);
                assert_eq!(Some("http".to_string()), config.api_protocol);
                assert_eq!(Some("dev".to_string()), config.pipeline);
                assert_eq!(Some(String::from("127.0.0.1")), config.server);
                assert_eq!(None, config.organization);
                assert_eq!(Some(true), config.non_interactive);
                assert_eq!(Some(true), config.auto_bump);
                assert_eq!(
                    Some("/path/to/my/custom/config.json".to_string()),
                    config.config_json
                );
                assert_eq!(Some(true), config.saml);
                assert_eq!(Some(true), config.fips);
                assert_eq!(Some("55555".to_string()), config.fips_git_port);
                assert_eq!(Some(true), config.a2_mode);
            }
            Err(e) => panic!("Failed to parse: {:?}", e.detail),
        }
    }

    #[test]
    fn test_api_url_with_port() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        conf.api_port = Some("2112".to_string());
        assert_eq!(
            "127.0.0.1:2112".to_string(),
            conf.api_host_and_port().unwrap()
        );
    }

    #[test]
    fn test_api_url_without_port() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        assert!(conf.api_port.is_none());
        assert_eq!("127.0.0.1".to_string(), conf.api_host_and_port().unwrap());
    }

    #[test]
    fn test_api_url_without_server() {
        let conf = Config::default();
        assert!(conf.server.is_none());
        assert!(conf.api_host_and_port().is_err());
    }

    #[test]
    fn test_base_resource_with_port_without_a2() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        conf.api_port = Some("2112".to_string());
        assert_eq!(
            "127.0.0.1:2112".to_string(),
            conf.api_base_resource().unwrap()
        );
    }


    #[test]
    fn test_api_base_resource_with_port_with_a2() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        conf.a2_mode = Some(true);
        conf.api_port = Some("2112".to_string());
        assert_eq!(
            "127.0.0.1:2112/workflow".to_string(),
            conf.api_base_resource().unwrap()
        );
    }

    #[test]
    fn test_api_base_resource_without_port_with_a2() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        conf.a2_mode = Some(true);
        assert!(conf.api_port.is_none());
        assert_eq!("127.0.0.1/workflow".to_string(), conf.api_base_resource().unwrap());
    }

    #[test]
    fn test_git_url_with_default_port() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        conf.user = Some("user".to_string());
        conf.enterprise = Some("ent".to_string());
        conf.organization = Some("org".to_string());
        conf.project = Some("proj".to_string());
        assert_eq!(
            "ssh://user@ent@127.0.0.1:8989/ent/org/proj".to_string(),
            conf.delivery_git_ssh_url().unwrap()
        );
    }

    #[test]
    fn test_git_url_with_port() {
        let mut conf = Config::default();
        conf.server = Some("127.0.0.1".to_string());
        conf.user = Some("user".to_string());
        conf.enterprise = Some("ent".to_string());
        conf.organization = Some("org".to_string());
        conf.project = Some("proj".to_string());
        conf.git_port = Some("2112".to_string());
        assert_eq!(
            "ssh://user@ent@127.0.0.1:2112/ent/org/proj".to_string(),
            conf.delivery_git_ssh_url().unwrap()
        );
    }

    #[test]
    fn test_git_url_without_server() {
        let conf = Config::default();
        assert!(conf.server.is_none());
        assert!(conf.delivery_git_ssh_url().is_err());
    }
}
