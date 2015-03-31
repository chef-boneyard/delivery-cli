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

use errors::DeliveryError;
use git;
use utils::mkdir_recursive;
use utils::path_join_many::PathJoinMany;
use utils::say::{say, sayln};

#[derive(RustcEncodable, Clone)]
pub struct DeliveryConfig {
    pub version: String,
    pub build_cookbook: HashMap<String, String>,
    pub skip_phases: Vec<String>,
}

impl Default for DeliveryConfig {
    fn default() -> DeliveryConfig {
        let mut build_cookbook = HashMap::new();
        build_cookbook.insert("name".to_string(),
                              "<your build cookbook name>".to_string());
        build_cookbook.insert("path".to_string(),
                              "<relative path from project root>".to_string());
        DeliveryConfig {
            version: "2".to_string(),
            build_cookbook: build_cookbook,
            skip_phases: Vec::new()
        }
    }
}

impl DeliveryConfig {
    pub fn init(proj_path: &PathBuf,
                proj_type_in: &str) -> Result<(), DeliveryError> {
        if DeliveryConfig::config_file_exists(proj_path) {
            debug!("Delivery config file already exists, skipping");
            return Ok(())
        }
        debug!("proj_path: {:?}\nproj_type_in: {:?}",
               proj_path, proj_type_in);
        let proj_type = if proj_type_in.is_empty() {
            if proj_path.join_many(&["metadata.rb"]).is_file() {
                "cookbook"
            } else {
                "other"
            }
        } else {
            proj_type_in
        };
        debug!("Creating a new config file for type: {}", proj_type);

        let mut config = DeliveryConfig::default();
        if proj_type == "cookbook" {
            let deliv_truck_git =
                "https://github.com/opscode-cookbooks/delivery-truck.git";
            let mut build_cookbook = HashMap::new();
            build_cookbook.insert("name".to_string(), "delivery-truck".to_string());
            build_cookbook.insert("git".to_string(), deliv_truck_git.to_string());
            build_cookbook.insert("branch".to_string(), "master".to_string());
            config.build_cookbook = build_cookbook;
            for phase in &["smoke", "security", "quality"] {
                config.skip_phases.push(phase.to_string())
            }
        }

        try!(config.write_file(proj_path));
        let config_path = DeliveryConfig::config_file_path(proj_path);
        let config_path_str = &config_path.to_str().unwrap();
        try!(git::git_command(&["checkout", "-b", "add-delivery-config"], proj_path));
        try!(git::git_command(&["add", &config_path_str], proj_path));
        try!(git::git_command(&["commit", "-m", "Add Delivery config"], proj_path));
        Ok(())
    }

    fn config_file_path(proj_path: &PathBuf) -> PathBuf {
        proj_path.join_many(&[".delivery", "config.json"])
    }

    fn config_file_exists(proj_path: &PathBuf) -> bool {
        DeliveryConfig::config_file_path(proj_path).is_file()
    }

    fn write_file(&self, proj_path: &PathBuf) -> Result<(), DeliveryError> {
        let write_dir = proj_path.join_many(&[".delivery"]);
        if !write_dir.is_dir() {
            try!(mkdir_recursive(&write_dir));
        }
        let write_path = DeliveryConfig::config_file_path(proj_path);
        say("white", "Writing configuration to ");
        sayln("yellow", &format!("{}", write_path.display()));
        let mut f = try!(File::create(&write_path));
        let json_obj = json::as_pretty_json(&self);
        let json_string = format!("{}", json_obj);
        sayln("magenta", "New delivery configuration");
        sayln("magenta", "--------------------------");
        say("white", &json_string);
        try!(f.write_all(json_string.as_bytes()));
        Ok(())
    }
}
