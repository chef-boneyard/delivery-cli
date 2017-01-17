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
use std::fmt::{Display, Formatter, Error};
use toml;
use types::DeliveryResult;
use utils;
use utils::path_join_many::PathJoinMany;

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct ProjectToml {
    pub remote_file: Option<String>,
    pub local_phases: Option<LocalPhases>
}

#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub struct LocalPhases {
    pub unit: Option<String>,
    pub lint: Option<String>,
    pub syntax: Option<String>,
    pub provision: Option<String>,
    pub deploy: Option<String>,
    pub smoke: Option<String>,
    pub functional: Option<String>,
    pub cleanup: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Unit,
    Lint,
    Syntax,
    Provision,
    Deploy,
    Smoke,
    Functional,
    Cleanup,
}

#[derive(Clone, Debug)]
pub enum Stage {
    Verify,
    Acceptance,
    All,
}

// Modify how we display this enum so we can print the phases
// with lowercases instead of capital letter, see command/local.rs
impl Display for Phase {
    fn fmt(&self, f:&mut Formatter) -> Result<(), Error> {
        match *self {
            Phase::Unit => write!(f, "unit"),
            Phase::Lint => write!(f, "lint"),
            Phase::Syntax => write!(f, "syntax"),
            Phase::Provision => write!(f, "provision"),
            Phase::Deploy => write!(f, "deploy"),
            Phase::Smoke => write!(f, "smoke"),
            Phase::Functional => write!(f, "funtional"),
            Phase::Cleanup => write!(f, "cleanup"),
        }
    }
}

impl Stage {
    pub fn phases(&self) -> Vec<Phase> {
        match *self {
            Stage::Verify => vec![
                Phase::Lint,
                Phase::Syntax,
                Phase::Unit,
            ],
            Stage::Acceptance => vec![
                Phase::Provision,
                Phase::Deploy,
                Phase::Smoke,
                Phase::Functional,
                Phase::Cleanup,
            ],
            Stage::All => vec![
                Phase::Lint,
                Phase::Syntax,
                Phase::Unit,
                Phase::Provision,
                Phase::Deploy,
                Phase::Smoke,
                Phase::Functional,
                Phase::Cleanup,
            ],
        }
    }
}

impl Default for ProjectToml {
    fn default() -> Self {
        ProjectToml {
            remote_file: None,
            local_phases: Some(LocalPhases {
                unit: None,
                lint: None,
                syntax: None,
                provision: None,
                deploy: None,
                smoke: None,
                functional: None,
                cleanup: None
            })
        }
    }
}

impl ProjectToml {
    pub fn load_toml(remote_toml: Option<&str>) -> DeliveryResult<ProjectToml> {
        if remote_toml.is_some() {
            let url = remote_toml.unwrap().clone();
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
                debug!("Content Remote project.toml: {:?}", toml);
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

    pub fn local_phase(&self, phase: Option<Phase>) -> DeliveryResult<Option<String>> {
        if let Some(p) = phase { 
            match self.local_phases {
                Some(ref phases) => {
                    match p {
                        Phase::Unit       => Ok(phases.unit.clone()),
                        Phase::Lint       => Ok(phases.lint.clone()),
                        Phase::Syntax     => Ok(phases.syntax.clone()),
                        Phase::Provision  => Ok(phases.provision.clone()),
                        Phase::Deploy     => Ok(phases.deploy.clone()),
                        Phase::Smoke      => Ok(phases.smoke.clone()),
                        Phase::Functional => Ok(phases.functional.clone()),
                        Phase::Cleanup    => Ok(phases.cleanup.clone()),
                    }
                },
                None => Err(DeliveryError{ kind: Kind::LocalPhasesNotFound, detail: None })
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
    pub use super::{ProjectToml, Phase, Stage};

    #[test]
    fn test_project_toml_with_defaults_plus_overrides() {
        let p_toml= ProjectToml::default();
        let unit = "mvn test".to_string();
        p_toml.local_phases.map(|mut phases| {
            // default is empty phases
            assert_eq!(None, phases.unit);
            // But if we fill them in
            phases.unit = Some(unit.clone());
            assert_eq!(unit, phases.unit.unwrap());
        });
    }

    #[test]
    fn test_parse_config_that_could_be_empty() {
        let  toml = r#"
# Now everything is optional so we can have an empty file
"#;
        let project_toml = ProjectToml::parse_config(toml);
        assert!(project_toml.is_ok());
    }

    #[test]
    fn test_local_phase_accessor() {
        let p_toml= ProjectToml::default();
        // Test failure - When None it must throw an Err()
        assert!(p_toml.local_phase(None).is_err());
        // If one works all of them does :)
        assert_eq!(p_toml.local_phase(Some(Phase::Unit)).unwrap(),
                   p_toml.local_phases.unwrap().unit);
    }

    #[test]
    fn test_stages_phases() {
        let verify = Stage::Verify;
        assert!(verify.phases().contains(&Phase::Syntax));
        assert!(verify.phases().contains(&Phase::Unit));
        assert!(verify.phases().contains(&Phase::Lint));

        let acceptance = Stage::Acceptance;
        assert!(acceptance.phases().contains(&Phase::Smoke));
        assert!(acceptance.phases().contains(&Phase::Deploy));
        assert!(acceptance.phases().contains(&Phase::Provision));
        assert!(acceptance.phases().contains(&Phase::Functional));
        assert!(acceptance.phases().contains(&Phase::Cleanup));

        let all = Stage::All;
        assert!(all.phases().contains(&Phase::Syntax));
        assert!(all.phases().contains(&Phase::Unit));
        assert!(all.phases().contains(&Phase::Lint));
        assert!(all.phases().contains(&Phase::Smoke));
        assert!(all.phases().contains(&Phase::Deploy));
        assert!(all.phases().contains(&Phase::Provision));
        assert!(all.phases().contains(&Phase::Functional));
        assert!(all.phases().contains(&Phase::Cleanup));
    }

    mod when_project_toml {
        pub use super::{ProjectToml, Phase};
        mod is_well_configured {
            fn toml<'a>() -> &'a str {
                r#"
                # This file is coming from chefdk (chef generate build-cookbook)
                [local_phases]
                unit = "rspec spec/"
                lint = "cookstyle"
                syntax = "foodcritic . --exclude spec -f any"
                provision = "chef exec kitchen create"
                deploy = "chef exec kitchen converge"
                smoke = "chef exec kitchen verify"
                cleanup = "chef exec kitchen destroy"
                "#
            }

            #[test]
            fn parse_project_config() {
                let project_toml = super::ProjectToml::parse_config(toml());
                match project_toml {
                    Ok(p_toml) => {
                        p_toml.local_phases.map(|phases| {
                            assert_eq!("rspec spec/".to_string(), phases.unit.unwrap());
                            assert_eq!("cookstyle".to_string(), phases.lint.unwrap());
                            assert_eq!("foodcritic . --exclude spec -f any".to_string(),
                                        phases.syntax.unwrap());
                            assert_eq!("chef exec kitchen create".to_string(),
                                        phases.provision.unwrap());
                            assert_eq!("chef exec kitchen converge".to_string(),
                                        phases.deploy.unwrap());
                            assert_eq!("chef exec kitchen verify".to_string(),
                                        phases.smoke.unwrap());
                            assert_eq!(None, phases.functional);
                            assert_eq!("chef exec kitchen destroy".to_string(),
                                        phases.cleanup.unwrap());
                        });
                    },
                    Err(e) => {
                        panic!("Failed to parse: {:?}", e.detail)
                    }
                }
            }
        }

        mod is_partially_configured {
            fn toml<'a>() -> &'a str {
                 r#"
                # Here we just define three phases
                [local_phases]
                unit = "rspec spec/"
                lint = "cookstyle"
                syntax = "something"
                "#
            }

            #[test]
            fn parse_project_config() {
                let project_toml = super::ProjectToml::parse_config(toml());
                match project_toml {
                    Ok(p_toml) => {
                        p_toml.local_phases.map(|phases| {
                            assert_eq!("rspec spec/".to_string(), phases.unit.unwrap());
                            assert_eq!("cookstyle".to_string(), phases.lint.unwrap());
                            assert_eq!("something".to_string(), phases.syntax.unwrap());
                            // The rest should be defined as None
                            // but we shouldn't fail parsing the file
                            assert!(phases.provision.is_none());
                            assert!(phases.deploy.is_none());
                            assert!(phases.smoke.is_none());
                            assert!(phases.functional.is_none());
                            assert!(phases.cleanup.is_none());
                        });
                    },
                    Err(e) => {
                        panic!("Failed to parse: {:?}", e.detail)
                    }
                }
            }
        }

        mod points_to_a_remote_file {
            fn toml<'a>() -> &'a str {
                r#"
                # No [local_phases]
                # Just point to a remote_file
                remote_file = "url"
                "#
            }

            #[test]
            fn parse_project_config() {
                let project_toml = super::ProjectToml::parse_config(toml());
                match project_toml {
                    Ok(p_toml) => {
                        assert_eq!("url".to_string(), p_toml.remote_file.unwrap());
                        // local_phases should be defined as None
                        assert!(p_toml.local_phases.is_none());
                    },
                    Err(e) => {
                        panic!("Failed to parse: {:?}", e.detail)
                    }
                }
            }
        }

        mod is_misconfigured {
        }
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
            assert_eq!("echo local-unit".to_string(), local_toml.local_phases.unwrap().unit.unwrap());
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
            assert_eq!("echo remote-unit".to_string(),
                       remote_toml.local_phases.unwrap().unit.unwrap());
        }
    }
}
