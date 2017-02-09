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
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use rustc_serialize::json;
use errors::{DeliveryError, Kind};
use types::DeliveryResult;
use git;
use utils::{walk_tree_for_path, read_file, copy_recursive, file_needs_updated};
use utils::path_join_many::PathJoinMany;

pub mod project;

#[derive(RustcEncodable, RustcDecodable, Clone)]
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
// This structure has two main files;
//   * version - The config version
//   * filters - Specific filters to search for nodes for each phase
//
// Example:
//  "job_dispatch": {
//    "version": "v2",
//    "filters": {
//      "unit": [
//        {
//          "platform_family": ["debian"],
//          "platform_version": ["12.04"]
//        },
//        {
//          "platform_family": ["rhel"]
//        }
//      ],
//      "syntax": [
//        {
//          "platform_family": ["debian"],
//          "platform_version": ["12.04"]
//        },
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
#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct JobDispatch {
    pub version: String,
    pub filters: Option<HashMap<String, Vec<HashMap<String, Vec<String>>>>>,
}

impl Default for DeliveryConfig {
    fn default() -> DeliveryConfig {
        let mut build_cookbook = HashMap::new();
        build_cookbook.insert("name".to_string(),
                              "build_cookbook".to_string());
        build_cookbook.insert("path".to_string(),
                              ".delivery/build_cookbook".to_string());

        let job_dispatch = JobDispatch {
            version: "v2".to_string(),
            filters: None,
        };

        DeliveryConfig {
            version: "2".to_string(),
            build_cookbook: build_cookbook,
            skip_phases: Some(Vec::new()),
            build_nodes: None,
            job_dispatch: Some(job_dispatch),
            dependencies: Some(Vec::new()),
        }
    }
}

impl DeliveryConfig {
    /// Copy a provided `config.json` file to `.delivery/` of
    /// the project root path. Also verify that the config is
    /// valid and finally add/commit the changes.
    /// If the config already exists, skip this process.
    pub fn copy_config_file(config_f: &PathBuf,
                            proj_path: &PathBuf) -> DeliveryResult<Option<String>> {
        let write_path = DeliveryConfig::config_file_path(proj_path);

        // If a config.json already exists, check to see if it is exactly
        // the same as what we want to copy to it.
        if !try!(file_needs_updated(config_f, &write_path)) {
            return Ok(None)
        }

        try!(copy_recursive(config_f, &write_path));
        try!(DeliveryConfig::validate_config_file(proj_path));
        let content = try!(read_file(&write_path));
        Ok(Some(content))
    }

    pub fn git_add_commit_config(proj_path: &PathBuf) -> DeliveryResult<bool> {
        let config_path = DeliveryConfig::config_file_path(proj_path);
        let config_path_str = &config_path.to_str().unwrap();
        try!(git::git_command(&["add", &config_path_str], proj_path));

        // Commit the changes made in .delivery but detect if nothing has changed,
        // if that is the case, we are Ok() to continue
        match git::git_commit("Adds custom Delivery config") {
          Ok(_) => Ok(true),
          Err(DeliveryError{ kind: Kind::EmptyGitCommit, .. }) => Ok(false),
          Err(e) => Err(e)
        }
    }

    pub fn config_file_path(proj_path: &PathBuf) -> PathBuf {
        proj_path.join_many(&[".delivery", "config.json"])
    }

    fn find_config_file(proj_path: &PathBuf) -> DeliveryResult<PathBuf> {
        match walk_tree_for_path(proj_path, ".delivery/config.json") {
            Some(p) => {
                debug!("found config: {:?}", p);
                Ok(p)
            },
            None => Err(DeliveryError{kind: Kind::MissingProjectConfig,
                                      detail: None})
        }
    }

    // Validate if the config.json is valid
    pub fn validate_config_file(proj_path: &PathBuf) -> DeliveryResult<bool> {
        let result = match DeliveryConfig::load_config(proj_path) {
            Ok(_) => Ok(true),
            Err(_) => {
                // Lets try as v1
                match DeliveryConfigV1::load_config(proj_path) {
                    Ok(_) => Ok(true),
                    Err(e) => Err(e)
                }
            }
        };
        // convert any error in a delivery error
        let boolean_result = try!(result);
        Ok(boolean_result)
    }

    // Load the .delivery/config.json into a DeliveryConfig object
    pub fn load_config(p_path: &PathBuf) -> DeliveryResult<DeliveryConfig> {
        let config_path = try!(DeliveryConfig::find_config_file(p_path));
        let mut config_file = try!(File::open(&config_path));
        let mut config_json = String::new();
        try!(config_file.read_to_string(&mut config_json));
        let json = try!(json::decode(&config_json));
        Ok(json)
    }
}

// v1 config, deprecated, but still supported
#[derive(RustcDecodable)]
pub struct DeliveryConfigV1 {
    pub version: String,
    pub build_cookbook: String,
    pub skip_phases: Option<Vec<String>>,
    pub build_nodes: Option<HashMap<String, Vec<String>>>
}

impl Default for DeliveryConfigV1 {
    fn default() -> DeliveryConfigV1 {
        DeliveryConfigV1 {
            version: "1".to_string(),
            build_cookbook: "./.delivery/build_cookbook".to_string(),
            skip_phases: Some(Vec::new()),
            build_nodes: Some(HashMap::new()),
        }
    }
}

impl DeliveryConfigV1 {
    pub fn load_config(p_path: &PathBuf) -> DeliveryResult<DeliveryConfigV1> {
        let config_path = try!(DeliveryConfig::find_config_file(p_path));
        let mut config_file = try!(File::open(&config_path));
        let mut config_json = String::new();
        try!(config_file.read_to_string(&mut config_json));
        let json = try!(json::decode(&config_json));
        Ok(json)
    }
}
