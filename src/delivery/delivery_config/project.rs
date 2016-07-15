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

use std::default::Default;
use std::path::PathBuf;
use utils::path_join_many::PathJoinMany;
use errors::{DeliveryError, Kind};
use types::DeliveryResult;
use rustc_serialize::Decodable;
use utils;
use toml;

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct ProjectToml {
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
    Cleanup
}

impl Default for ProjectToml {
    fn default() -> Self {
        ProjectToml {
            local_phases: LocalPhases {
                unit: String::from(""),
                lint: String::from(""),
                syntax: String::from(""),
                provision: String::from(""),
                deploy: String::from(""),
                smoke: String::from(""),
                cleanup: String::from("")
            }
        }
    }
}

impl ProjectToml {
    pub fn load_toml_file(proj_path: PathBuf) -> DeliveryResult<ProjectToml> {
        let toml_path = ProjectToml::toml_file_path(proj_path);
        try!(ProjectToml::validate_file(&toml_path));
        let toml = try!(utils::read_file(&toml_path));
        ProjectToml::parse_config(&toml)
    }

    pub fn local_phase(&self, phase: Option<Phase>) -> DeliveryResult<String> {
        if let Some(p) = phase { 
            match p {
                Phase::Unit      => Ok(self.local_phases.unit.clone()),
                Phase::Lint      => Ok(self.local_phases.lint.clone()),
                Phase::Syntax    => Ok(self.local_phases.syntax.clone()),
                Phase::Provision => Ok(self.local_phases.provision.clone()),
                Phase::Deploy    => Ok(self.local_phases.deploy.clone()),
                Phase::Smoke     => Ok(self.local_phases.smoke.clone()),
                Phase::Cleanup   => Ok(self.local_phases.cleanup.clone()),
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
    use super::{ProjectToml, Phase};

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
}
