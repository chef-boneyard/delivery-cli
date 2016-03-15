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

use errors::{DeliveryError, Kind};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use utils::read_file;
use utils::say::{say, sayln};
use utils::path_ext::is_file;
use git;
use project;
use regex::Regex;

// Return the path to the metadata.rb file
fn metadata_file(path: &PathBuf) -> String {
    let mut metadata = path.to_str().unwrap().to_string();
    metadata.push_str("/metadata.rb");
    return metadata
}

// Vefiry if the provided path is a cookbook, or not
fn is_cookbook(path: &PathBuf) -> bool {
    let meta_f = metadata_file(path);
    is_file(&Path::new(&meta_f))
}

// Extract the cookbook version from the provided metadata content
//
// This function expects you to read the metadata in advance and
// pass just the content of the file. From there it will read every
// line and search for the version to return it.
fn metadata_version_from(content: String) -> Result<String, DeliveryError> {
    for l in content.lines() {
        let r = Regex::new(r"version\s+'(?P<version>[0-9]*\.[0-9]*\.[0-9]*)'").unwrap();
        if let Some(version) = r.captures(l) {
            return Ok(version.name("version").unwrap().to_string())
        };
    };
    return Err(DeliveryError{ kind: Kind::MissingMetadataVersion, detail: None })
}

// Bump the metadata version, only if:
// * The project is a cookbook
// * The version hasn't been updated
//
// @param p_root [&PathBuf] The project root path
// @param pipeline [&str] Pipeline the change is targeting to
// @return ()
pub fn bump_version(p_root: &PathBuf, pipeline: &str) -> Result<(), DeliveryError> {
    if is_cookbook(&p_root) {
        let project = try!(project::project_from_cwd());
        say("white", "Project ");
        say("yellow", &project);
        sayln("white", " is a cookbook");
        sayln("white", "Validating version in metadata");

        let meta_f_p = PathBuf::from(metadata_file(&p_root));
        let current_v = try!(metadata_version_from(try!(read_file(&meta_f_p))));
        let t_file = [pipeline, ":metadata.rb"].concat();
        let pipeline_meta = try!(git::git_command(&["show", &t_file], &p_root));
        let pipeline_v = try!(metadata_version_from(pipeline_meta.stdout));

        if current_v == pipeline_v {
            say("yellow", "The version hasn't been updated (");
            say("red", &pipeline_v);
            sayln("yellow", ")");
            let s = pipeline_v.split(".");
            let vec: Vec<&str> = s.collect();
            let patch = vec[2].parse::<u32>().unwrap();
            let new_patch: String = (patch + 1).to_string();
            let new_version = [vec[0], ".", vec[1], ".", new_patch.trim()].concat();
            say("white", "Bumping version to: ");
            sayln("green", &new_version);

            let current_meta = try!(read_file(&meta_f_p));
            let new_meta = current_meta.replace(&*current_v, &*new_version);

            // Recreate the file and dump the processed contents to it
            let mut recreate_meta = try!(File::create(metadata_file(&p_root)));
            try!(recreate_meta.write_fmt(format_args!("{}", &new_meta)));

            let commit_msg = ["Bump version to ", new_version.trim()].concat();
            try!(git::git_command(&["add", &metadata_file(&p_root)], &p_root));
            try!(git::git_command(&["commit", "-m", &commit_msg], &p_root));
        } else {
            say("white", "Version already updated (");
            say("magenta", &pipeline_v);
            say("white", "/");
            say("green", &current_v);
            sayln("white", ")");
        }
    }
    Ok(())
}
