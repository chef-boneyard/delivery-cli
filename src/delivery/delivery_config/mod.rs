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

/// This module is responsible for handling the .delivery/config.json file

use std::collections::HashMap;
use std::default::Default;
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use errors::{DeliveryError, Kind};
use types::DeliveryResult;
use utils::{walk_tree_for_path, read_file, copy_recursive, file_needs_updated};
use utils::path_join_many::PathJoinMany;
use serde_json;
use serde_json::Value as SerdeJson;
use git;

pub mod project;

#[derive(Serialize, Deserialize, Clone)]
pub struct DeliveryConfig {
    pub version: String,
    pub build_cookbook: HashMap<String, String>,
    pub skip_phases: Option<Vec<String>>,
    pub build_nodes: Option<HashMap<String, Vec<String>>>,
    pub job_dispatch: Option<JobDispatch>,
    pub dependencies: Option<Vec<String>>,
}

// JobDispatch Struct
//
// This structure has two main fields;
//   * version - The config version
//   * filters - Specific search nodes filters for each phase
//
// Example:
//  "job_dispatch": {
//    "version": "v2",
//    "filters": {
//      "unit": [
//        {
//          "platform_family": ["debian"],
//          "platform_version": ["14.04"]
//        },
//        {
//          "platform_family": ["rhel"]
//        }
//      ],
//      "syntax": [
//        {
//          "platform_family": ["debian"],
//          "platform_version": ["14.04"]
//        },
//        {
//          "platform_family": ["rhel"]
//        }
//      ]
//    }
//  }
#[derive(Serialize, Deserialize, Clone)]
pub struct JobDispatch {
    pub version: String,
    pub filters: Option<HashMap<String, SerdeJson>>,
}

impl Default for JobDispatch {
    fn default() -> Self {
        JobDispatch {
            version: "v2".to_string(),
            filters: None,
        }
    }
}

impl Default for DeliveryConfig {
    fn default() -> Self {
        let mut build_cookbook = HashMap::new();
        build_cookbook.insert("name".to_string(),
                              "build_cookbook".to_string());
        build_cookbook.insert("path".to_string(),
                              ".delivery/build_cookbook".to_string());

        DeliveryConfig {
            version: "2".to_string(),
            build_cookbook: build_cookbook,
            skip_phases: Some(Vec::new()),
            build_nodes: None,
            job_dispatch: Some(JobDispatch::default()),
            dependencies: Some(Vec::new()),
        }
    }
}

// BuildCookbookLocation Enumarator
//
// The build_cokbook could be sourced from the following locations:
//   * Local       - On the project repo
//   * Git         - From a git server. (url)
//   * Supermarket - From a Supermarket Server
//   * Workflow    - From the Workflow Server
//   * ChefServer  - From the Chef Server
//
// Examples: https://docs.chef.io/config_json_delivery.html#examples
#[derive(Debug, PartialEq)]
pub enum BuildCookbookLocation {
    Local,
    Git,
    Supermarket,
    Workflow,
    ChefServer,
}

impl DeliveryConfig {
    /// Return the build_cookbook location
    ///
    /// Searches for the right field inside the build_cookbook HashMap
    /// and translates it to a BuildCookbookLocation Enum, if none of
    /// the possible entries exist, throws a `Err()`
    ///
    /// # Examples
    ///
    /// ```
    /// use delivery::delivery_config::{DeliveryConfig, BuildCookbookLocation};
    ///
    /// let config   = DeliveryConfig::default();
    /// let location = config.build_cookbook_location().unwrap();
    /// assert_eq!(BuildCookbookLocation::Local, location);
    /// ```
    pub fn build_cookbook_location(&self) -> DeliveryResult<BuildCookbookLocation> {
        for (key, _) in self.build_cookbook.iter() {
            match key.as_str() {
                "path"          => return Ok(BuildCookbookLocation::Local),
                "git"           => return Ok(BuildCookbookLocation::Git),
                "supermarket"   => return Ok(BuildCookbookLocation::Supermarket),
                "enterprise"    => return Ok(BuildCookbookLocation::Workflow),
                "server"        => return Ok(BuildCookbookLocation::ChefServer),
                _               => continue,
            }
        }
        Err(DeliveryError{ kind: Kind::NoValidBuildCookbook, detail: None })
    }

    /// Get the content of a specific build_cookbook field
    ///
    /// The build_cookbook is defined as a HashMap that we can easily extract
    /// the content of a particular `key`, this will reduce complexity and code
    ///
    /// # Examples
    ///
    /// ```
    /// use delivery::delivery_config::DeliveryConfig;
    ///
    /// let config        = DeliveryConfig::default();
    /// let cookbook_name = config.build_cookbook_get("name").unwrap();
    /// assert_eq!("build_cookbook".to_string(), cookbook_name);
    /// ```
    pub fn build_cookbook_get(&self, key: &str) -> DeliveryResult<String> {
        self.build_cookbook.get(key)
            .ok_or(DeliveryError{
                kind: Kind::MissingBuildCookbookField,
                detail: Some(format!("Unable to find '{}' field.", key).to_string())
            }).map(|s| { s.to_owned() })
    }

    // Return the build_cookbook name
    //
    // A valid Delivery V2 config should always have a `name` entry
    // inside the build_cookbook HashMap.
    pub fn build_cookbook_name(&self) -> DeliveryResult<String> {
        self.build_cookbook_get("name")
    }

    /// Copy a provided `config.json` file to `.delivery/` of
    /// the project root path. Also verify that the config is
    /// valid and finally add/commit the changes.
    /// If the config already exists, skip this process.
    pub fn copy_config_file<P>(config_f: P, proj_path: P) -> DeliveryResult<Option<String>>
            where P: AsRef<Path> + Debug {
        let write_path = DeliveryConfig::config_file_path(&proj_path);

        // If a config.json already exists, check to see if it is exactly
        // the same as what we want to copy to it.
        if !file_needs_updated(&config_f, &write_path)? {
            return Ok(None)
        }

        try!(copy_recursive(&config_f, &write_path));
        try!(DeliveryConfig::validate_config_file(&proj_path));
        Ok(Some(read_file(&write_path)?))
    }

    pub fn git_add_commit_config<P>(proj_path: P) -> DeliveryResult<bool>
            where P: AsRef<Path> {
        let config_path = DeliveryConfig::config_file_path(&proj_path);
        let config_path_str = &config_path.to_str().unwrap();
        try!(git::git_command(&["add", &config_path_str], &proj_path));

        // Commit the changes made in .delivery but detect if nothing has changed,
        // if that is the case, we are Ok() to continue
        match git::git_commit("Adds custom Delivery config") {
          Ok(_) => Ok(true),
          Err(DeliveryError{ kind: Kind::EmptyGitCommit, .. }) => Ok(false),
          Err(e) => Err(e)
        }
    }

    // Returns the path of the `config.json` from the provided project path
    fn config_file_path<P>(p_path: P) -> PathBuf
            where P: AsRef<Path>  {
        p_path.as_ref().join_many(&[".delivery", "config.json"])
    }

    fn find_config_file<P>(proj_path: P) -> DeliveryResult<PathBuf>
            where P: AsRef<Path> {
        match walk_tree_for_path(proj_path, ".delivery/config.json") {
            Some(p) => {
                debug!("found config: {:?}", p);
                Ok(p)
            },
            None => Err(DeliveryError{kind: Kind::MissingProjectConfig,
                                      detail: None})
        }
    }

    // Validate the config.json file
    pub fn validate_config_file<P>(p_path: P) -> DeliveryResult<bool>
            where P: AsRef<Path> + Debug {
        DeliveryConfig::load_config(p_path).and(Ok(true))
    }

    // Load the .delivery/config.json into a DeliveryConfig object
    //
    // This fn is capable of loading the `config.json` from a provided
    // path, it will try to decode the config V2 (latest at the moment)
    // and if it is unable to do so, it will try to decode in V1
    pub fn load_config<P>(p_path: P) -> DeliveryResult<Self>
            where P: AsRef<Path> + Debug {
        debug!("Loading config.json into memory from path: {:?}", p_path);
        let config_json = read_file(&DeliveryConfig::find_config_file(&p_path)?)?;

        // Try to decode the config, but if you are unable to, try V1;
        // If you are still unable; just fail
        let json: DeliveryConfig = serde_json::from_str(&config_json).or_else( |e_v2| {
            debug!("Unable to parse DeliveryConfig: {}\nV2 Error: {}", config_json, e_v2);
            debug!("Attempting to load version: 1");
            let v1_config = DeliveryConfigV1::load_config(&p_path).or_else(|e_v1| {
                debug!("Unable to parse DeliveryConfigV1: {}", e_v1);
                // If we couldn't parse any version of the delivery config,
                // lets make sure we give the user the right error message
                Err(DeliveryError {
                    kind: Kind::DeliveryConfigParse,
                    detail: Some(format!(
                        "  version 2: {}\n  version 1: {}\n\nSee: \
                        https://docs.chef.io/config_json_delivery.html",
                        e_v2, e_v1).to_string()
                    ),
                })
            })?;
            v1_config.convert_to_v2()
        })?;
        Ok(json)
    }

}

// v1 config, deprecated, but still supported
#[derive(Deserialize)]
pub struct DeliveryConfigV1 {
    pub version: String,
    pub build_cookbook: String,
    pub skip_phases: Option<Vec<String>>,
    pub build_nodes: Option<HashMap<String, Vec<String>>>
}

impl Default for DeliveryConfigV1 {
    fn default() -> Self {
        DeliveryConfigV1 {
            version: "1".to_string(),
            build_cookbook: "./.delivery/build_cookbook".to_string(),
            skip_phases: Some(Vec::new()),
            build_nodes: Some(HashMap::new()),
        }
    }
}

impl DeliveryConfigV1 {
    // Load the .delivery/config.json into a DeliveryConfigV1 object
    pub fn load_config<P>(p_path: P) -> DeliveryResult<Self>
            where P: AsRef<Path> + Debug {
        let config_json = read_file(&DeliveryConfig::find_config_file(p_path)?)?;
        Ok(serde_json::from_str::<DeliveryConfigV1>(&config_json)?)
    }

    // Convert DeliveryConfigV1 to V2
    //
    // The big difference between V1 and V2 is that the build_cookbook field was
    // at first a simple String that pointed to either a build_cookbook path stored
    // locally or a simple name of the build_cookbook that would mean we would pull
    // it from the Chef Sever. In V2 instead we allows multiple locations including
    // `path` and `server` among others.
    //
    // This function will decode a V1 config and convert it into V2 properly
    fn convert_to_v2(self) -> DeliveryResult<DeliveryConfig> {
        let mut build_cookbook = HashMap::new();

        // Detect if the build_cookbook is stored locally or remotely
        if self.build_cookbook.contains("/") {
            // A local path, lets add the `path` field
            let cookbook_path = PathBuf::from(&self.build_cookbook);
            let cookbook_name = cookbook_path.file_name().ok_or(DeliveryError{
                kind: Kind::NoValidBuildCookbook,
                detail: Some("V1: Expected a valid path to a build_cookbook".to_string())
            })?;

            build_cookbook.insert(String::from("name"),
                                  cookbook_name
                                    .to_string_lossy()
                                    .into_owned());
            build_cookbook.insert(String::from("path"),
                                  cookbook_path
                                    .to_string_lossy()
                                    .into_owned());
        } else {
            // A build_cookbook name, load it from the `server`
            build_cookbook.insert(String::from("name"),
                                  self.build_cookbook);
            build_cookbook.insert(String::from("server"), String::from("true"));
        }

        Ok(
            // Instantiate a DeliveryConfig consuming `self` properties
            DeliveryConfig {
                // This is a config coming from V1, lets persist this
                version: "1".to_string(),
                build_cookbook: build_cookbook,
                skip_phases: self.skip_phases,
                build_nodes: self.build_nodes,
                job_dispatch: None,
                dependencies: None,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod v1 {
        use super::*;

        #[test]
        fn default() {
            let c_v1 = DeliveryConfigV1::default();
            assert_eq!(c_v1.version, "1".to_string());
            assert_eq!(c_v1.build_cookbook, "./.delivery/build_cookbook".to_string());
            assert_eq!(c_v1.skip_phases, Some(Vec::new()));
            assert_eq!(c_v1.build_nodes, Some(HashMap::new()));
        }
    }

    mod v2 {
        use super::*;

        #[test]
        fn default() {
            let mut build_cookbook = HashMap::new();
            build_cookbook.insert("name".to_string(),
                                  "build_cookbook".to_string());
            build_cookbook.insert("path".to_string(),
                                  ".delivery/build_cookbook".to_string());
            let c_v2 = DeliveryConfig::default();
            assert_eq!(c_v2.version, "2".to_string());
            assert_eq!(c_v2.skip_phases, Some(Vec::new()));
            assert_eq!(c_v2.job_dispatch.unwrap().version, "v2".to_string());
            assert_eq!(c_v2.dependencies, Some(Vec::new()));
            assert_eq!(c_v2.build_cookbook, build_cookbook);
            assert!(c_v2.build_nodes.is_none());
        }

        mod job_dispatch {
            use super::*;

            #[test]
            fn default() {
                let job_d = JobDispatch::default();
                assert_eq!(job_d.version, "v2".to_string());
                assert!(job_d.filters.is_none());
            }

            #[test]
            fn complex_config() {
                let mut complex = JobDispatch::default();
                let filter = json!([
                    {
                        "platform_family": ["debian"],
                        "platform_version": ["14.04"],
                    },
                    {
                        "platform_family": ["rhel"],
                    }
                ]);
                let mut filters = HashMap::new();
                filters.insert("unit".to_string(), filter.clone());
                filters.insert("syntax".to_string(), filter.clone());
                filters.insert("publish".to_string(), filter.clone());

                complex.filters = Some(filters);
                assert!(complex.filters.clone().unwrap().get("unit").is_some());
                assert!(complex.filters.clone().unwrap().get("syntax").is_some());
                assert!(complex.filters.clone().unwrap().get("publish").is_some());

                // Extract the platform_family content of the filter1 for the unit phase
                assert!(complex.filters.clone().unwrap().get("unit")
                        .unwrap()[0].get("platform_family").is_some());
                // Extract the platform_version content of the filter1 for the syntax phase
                assert_eq!(complex.filters.clone().unwrap().get("syntax")
                        .unwrap()[0].get("platform_version").unwrap()[0],
                        String::from("14.04"));
                // Extract the platform_family content of the filter2 for the publish phase
                assert_eq!(complex.filters.clone().unwrap().get("publish")
                        .unwrap()[1].get("platform_family").unwrap()[0],
                        String::from("rhel"));

                // Filter for deploy phase doesn't exist
                assert!(complex.filters.clone().unwrap().get("deploy").is_none());
            }

            #[test]
            fn simple_filter() {
                let mut simple = JobDispatch::default();
                let filter = json!({
                    "platform_family": ["debian"],
                    "platform_version": ["14.04"],
                });
                let mut filters = HashMap::new();
                filters.insert("default".to_string(), filter);

                simple.filters = Some(filters);

                // Extract the platform_version content of the filter for the default phase
                assert_eq!(simple.filters.clone().unwrap().get("default")
                        .unwrap().get("platform_version").unwrap()[0],
                        String::from("14.04"));

                // Extracting a key that doesn't exist will return None
                assert!(simple.filters.clone().unwrap().get("default")
                        .unwrap().get("platform").is_none());

                // Filter for deploy phase doesn't exist
                assert!(simple.filters.clone().unwrap().get("deploy").is_none());
            }

            mod build_cookbook {
                use super::*;

                fn blank_bk_with_name() -> HashMap<String, String> {
                    let mut bk = HashMap::new();
                    bk.insert("name".to_string(), "cook_nice".to_string());
                    bk
                }

                fn chef_server() -> HashMap<String, String> {
                    let mut chef = blank_bk_with_name();
                    chef.insert("server".to_string(), "true".to_string());
                    chef
                }

                fn supermarket() -> HashMap<String, String> {
                    let mut supermarket = blank_bk_with_name();
                    supermarket.insert("supermarket".to_string(), "true".to_string());
                    supermarket
                }

                fn workflow() -> HashMap<String, String> {
                    let mut workflow = blank_bk_with_name();
                    workflow.insert("enterprise".to_string(), "mx".to_string());
                    workflow.insert("organization".to_string(), "df".to_string());
                    workflow
                }

                fn git() -> HashMap<String, String> {
                    let mut git = blank_bk_with_name();
                    git.insert("git".to_string(),
                        "git@github.com:marvel/cerebro.git".to_string());
                    git
                }

                mod location {
                    use super::*;

                    #[test]
                    fn default() {
                        let config = DeliveryConfig::default();
                        assert_eq!(BuildCookbookLocation::Local,
                                   config.build_cookbook_location().unwrap());
                    }

                    #[test]
                    fn chef_server() {
                        let mut config = DeliveryConfig::default();
                        config.build_cookbook = super::chef_server();
                        assert_eq!(BuildCookbookLocation::ChefServer,
                                   config.build_cookbook_location().unwrap());
                    }

                    #[test]
                    fn supermarket() {
                        let mut config = DeliveryConfig::default();
                        config.build_cookbook = super::supermarket();
                        assert_eq!(BuildCookbookLocation::Supermarket,
                                   config.build_cookbook_location().unwrap());
                    }

                    #[test]
                    fn workflow() {
                        let mut config = DeliveryConfig::default();
                        config.build_cookbook = super::workflow();
                        assert_eq!(BuildCookbookLocation::Workflow,
                                   config.build_cookbook_location().unwrap());
                    }

                    #[test]
                    fn git() {
                        let mut config = DeliveryConfig::default();
                        config.build_cookbook = super::git();
                        assert_eq!(BuildCookbookLocation::Git,
                                   config.build_cookbook_location().unwrap());
                    }

                    #[test]
                    fn failure_invalid_build_cookbook() {
                        let mut config = DeliveryConfig::default();
                        config.build_cookbook = super::blank_bk_with_name();
                        assert!(config.build_cookbook_location().is_err());
                    }
                }

                #[test]
                fn get() {
                    let mut config = DeliveryConfig::default();
                    config.build_cookbook = workflow();
                    assert_eq!("cook_nice".to_string(),
                                config.build_cookbook_get("name").unwrap());
                    assert_eq!("mx".to_string(),
                                config.build_cookbook_get("enterprise").unwrap());
                    assert_eq!("df".to_string(),
                                config.build_cookbook_get("organization").unwrap());

                    // This doesn't exist :-)
                    assert!(config.build_cookbook_get("so").is_err());
                    assert!(config.build_cookbook_get("much").is_err());
                    assert!(config.build_cookbook_get("fun").is_err());
                }

                #[test]
                fn name() {
                    let config = DeliveryConfig::default();
                    assert_eq!("build_cookbook".to_string(),
                                config.build_cookbook_name().unwrap());
                }
            }
        }
    }
}
