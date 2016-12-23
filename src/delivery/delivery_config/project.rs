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

/// Delivery Prototype
///
/// This module is responsible for handling the .delivery/project.toml file
/// that is currently a prototype for local phases executioin. This file can
/// be configurable and it doesn't conflict with the existing config.json

use errors::{DeliveryError, Kind};
use hyper::Client as HyperClient;
use project;
use rustc_serialize::Decodable;
use std::default::Default;
use std::io::Read;
use std::path::PathBuf;
use toml;
use types::DeliveryResult;
use utils;
use utils::path_join_many::PathJoinMany;

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct ProjectToml {
    pub remote_file: Option<String>,
    pub local_phases: LocalPhases
}

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct LocalPhases {
    pub unit: String,
    pub lint: String,
    pub syntax: String,
    pub provision: String,
    pub deploy: String,
    pub smoke: String,
    pub functional: String,
    pub cleanup: String,
}

#[derive(Clone, Debug)]
pub enum Phase {
    Unit,
    Lint,
    Syntax,
    Provision,
    Deploy,
    Smoke,
    Functional,
    Cleanup
}

impl Default for ProjectToml {
    fn default() -> Self {
        ProjectToml {
            remote_file: None,
            local_phases: LocalPhases {
                unit: String::from(""),
                lint: String::from(""),
                syntax: String::from(""),
                provision: String::from(""),
                deploy: String::from(""),
                smoke: String::from(""),
                functional: String::from(""),
                cleanup: String::from("")
            }
        }
    }
}

impl ProjectToml {
    pub fn load_toml(remote_toml: &str) -> DeliveryResult<ProjectToml> {
        if ! remote_toml.is_empty() {
            let url = remote_toml.clone();
            return ProjectToml::load_toml_remote(url)
        }

        let path = ProjectToml::toml_file_path(project::project_path());
        let project_toml = try!(ProjectToml::load_toml_file(path));

        match project_toml.remote_file {
            Some(url) => ProjectToml::load_toml_remote(&url),
            None => Ok(project_toml)
        }
    }

    fn load_toml_file(toml_path: PathBuf) -> DeliveryResult<ProjectToml> {
        debug!("Loading local project.toml from {:?}", toml_path);
        try!(ProjectToml::validate_file(&toml_path));
        let toml = try!(utils::read_file(&toml_path));
        ProjectToml::parse_config(&toml)
    }

    fn load_toml_remote(toml_url: &str) -> DeliveryResult<ProjectToml> {
        debug!("Loading remote project.toml from {:?}", toml_url);
        let client = HyperClient::new();
        match client.get(toml_url).send() {
            Ok(mut resp) => {
                let mut toml = String::new();
                try!(resp.read_to_string(&mut toml));
                ProjectToml::parse_config(&toml)
            },
            Err(e) => {
                Err(DeliveryError{
                    kind: Kind::HttpError(e),
                    detail: None
                })
            }
        }
    }

    pub fn local_phase(&self, phase: Option<Phase>) -> DeliveryResult<String> {
        if let Some(p) = phase { 
            match p {
                Phase::Unit       => Ok(self.local_phases.unit.clone()),
                Phase::Lint       => Ok(self.local_phases.lint.clone()),
                Phase::Syntax     => Ok(self.local_phases.syntax.clone()),
                Phase::Provision  => Ok(self.local_phases.provision.clone()),
                Phase::Deploy     => Ok(self.local_phases.deploy.clone()),
                Phase::Smoke      => Ok(self.local_phases.smoke.clone()),
                Phase::Functional => Ok(self.local_phases.functional.clone()),
                Phase::Cleanup    => Ok(self.local_phases.cleanup.clone()),
            }
        } else {
            Err(DeliveryError{ kind: Kind::PhaseNotFound, detail: None })
       }
    }

    fn toml_file_path(proj_path: PathBuf) -> PathBuf {
        proj_path.join_many(&[".delivery", "project.toml"])
    }

    fn parse_config(toml: &str) -> DeliveryResult<ProjectToml> {
        let mut parser = toml::Parser::new(toml);
        match parser.parse() {
            Some(value) => { 
                ProjectToml::decode_toml(toml::Value::Table(value))
            },
            None => {
                Err(DeliveryError{
                    kind: Kind::ConfigParse,
                    detail: Some(
                        format!("Unable to parse .delivery/project.toml: {:?}",
                        parser.errors)
                    )
                })
            }
        }
    }

    fn validate_file(toml_path: &PathBuf) -> DeliveryResult<()> {
        if toml_path.exists() {
            Ok(())
        } else {
            Err(DeliveryError{ 
                kind: Kind::MissingConfigFile,
                detail: Some(
                    format!("The .delivery/project.toml file was not found.\n\n\
                            You can generate this file using the command:\n\
                            \tchef generate build-cookbook [NAME]")
                )
            })
        }
    }

    fn decode_toml(table: toml::Value) -> DeliveryResult<ProjectToml> {
        let mut decoder = toml::Decoder::new(table);
        let project_config = try!(ProjectToml::decode(&mut decoder));
        Ok(project_config) 
    }
}


#[cfg(test)]
mod tests {
    pub use super::{ProjectToml, Phase};

    #[test]
    fn test_project_toml_with_defaults_plus_overrides() {
        // default is empty phases
        let mut p_toml= ProjectToml::default();
        assert_eq!("".to_string(), p_toml.local_phases.unit);
        // But if we fill them in
        p_toml.local_phases.unit = "mvn test".to_string();
        assert_eq!("mvn test".to_string(), p_toml.local_phases.unit);
    }

    #[test]
    fn test_parse_project_config() {
        let  toml = r#"
# This file is coming from chefdk (chef generate build-cookbook)
[local_phases]
unit = "rspec spec/"
lint = "cookstyle"
syntax = "foodcritic . --exclude spec -f any"
provision = "chef exec kitchen create"
deploy = "chef exec kitchen converge"
smoke = "chef exec kitchen verify"
functional = ""
cleanup = "chef exec kitchen destroy"
"#;
        let project_toml = ProjectToml::parse_config(toml);
        match project_toml {
            Ok(p_toml) => {
                assert_eq!("rspec spec/".to_string(), p_toml.local_phases.unit);
                assert_eq!("cookstyle".to_string(), p_toml.local_phases.lint);
                assert_eq!("foodcritic . --exclude spec -f any".to_string(), p_toml.local_phases.syntax);
                assert_eq!("chef exec kitchen create".to_string(), p_toml.local_phases.provision);
                assert_eq!("chef exec kitchen converge".to_string(), p_toml.local_phases.deploy);
                assert_eq!("chef exec kitchen verify".to_string(), p_toml.local_phases.smoke);
                assert_eq!("".to_string(), p_toml.local_phases.functional);
                assert_eq!("chef exec kitchen destroy".to_string(), p_toml.local_phases.cleanup);
            },
            Err(e) => {
                panic!("Failed to parse: {:?}", e.detail)
            }
        }
    }

    #[test]
    fn test_parse_config_error_when_toml_file_is_misconfigured() {
        let  toml = r#"
# Here it is missing the key [local_phases]
lint = "cookstyle"
syntax = "something"
"#;
        let project_toml = ProjectToml::parse_config(toml);
        match project_toml {
            Ok(_) => {
                panic!("This shouldn't return an Ok() - verify test")
            },
            Err(e) => {
                let msg = String::from("expected a field: expected a section \
                            for the key `local_phases`");
                assert_eq!(Some(msg), e.detail);
            }
        }
    }

    #[test]
    fn test_local_phase_accessor() {
        let p_toml= ProjectToml::default();
        // If one works all of them does :)
        assert_eq!(p_toml.local_phase(Some(Phase::Unit)).unwrap(), p_toml.local_phases.unit);
        // Test failure - When None it must throw an Err()
        assert!(p_toml.local_phase(None).is_err());
    }

    mod toml_file_path {
        pub use super::{ProjectToml};
        use std::path::{PathBuf};

        #[test]
        fn returns_path_to_toml() {
            let project_path = PathBuf::from("/tmp");
            let expected = PathBuf::from("/tmp/.delivery/project.toml");
            let actual = ProjectToml::toml_file_path(project_path);
            assert_eq!(expected, actual);
        }
    }

    mod load_toml_file {
        pub use super::{ProjectToml};
        use std::path::{PathBuf};
        use std::fs::File;
        use std::io::Write;

        #[test]
        fn returns_toml_from_path() {
            let path = PathBuf::from("/tmp/local.toml");
            let toml = r#"
[local_phases]
unit = "echo local-unit"
lint = "echo local-lint"
syntax = "echo local-syntax"
provision = "echo local-provision"
deploy = "echo local-deploy"
smoke = "echo local-smoke"
functional = "echo local-functional"
cleanup = "echo local-cleanup"
"#;

            let mut file = File::create(&path).expect("Unable to create local toml file");
            file.write_all(toml.as_bytes()).expect("Unable to write local toml file");

            let local_toml = ProjectToml::load_toml_file(path).unwrap();
            assert_eq!("echo local-unit".to_string(), local_toml.local_phases.unit);
        }
    }

    mod load_toml_remote {
        pub use super::{ProjectToml};
        use mockito::mock;

        #[test]
        fn returns_toml_from_url() {
            let url = "http://0.0.0.0:1234/toml-url";
            let toml = r#"
[local_phases]
unit = "echo remote-unit"
lint = "echo remote-lint"
syntax = "echo remote-syntax"
provision = "echo remote-provision"
deploy = "echo remote-deploy"
smoke = "echo remote-smoke"
functional = "echo remote-functional"
cleanup = "echo remote-cleanup"
"#;

            mock("GET", "/toml-url")
                .with_status(200)
                .with_header("content-type", "text/plain")
                .with_body(toml)
                .create();

            let remote_toml = ProjectToml::load_toml_remote(url).unwrap();
            assert_eq!("echo remote-unit".to_string(), remote_toml.local_phases.unit);
        }
    }
}
