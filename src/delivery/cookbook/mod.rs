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
use utils::{self, read_file};
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

// Verify if the provided path is a cookbook, or not
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

// Bump the patchset of the provided version
fn bump_patchset(version: String) -> Result<String, DeliveryError> {
    let s = version.split(".");
    let m_m_p: Vec<&str> = s.collect();
    let patch = try!(m_m_p[2].parse::<u32>());
    let new_patch: String = (patch + 1).to_string();
    let mut new_version: String = m_m_p[0].to_string();
    new_version.push_str(".");
    new_version.push_str(m_m_p[1]);
    new_version.push_str(".");
    new_version.push_str(&new_patch);
    Ok(new_version)
}

// Saves the new version to the metadata and commit the changes
fn save_version(metadata: &PathBuf, version: String) -> Result<(), DeliveryError> {
    let current_meta = try!(read_file(metadata));
    let current_version = try!(metadata_version_from(current_meta.clone()));
    let new_metadata = current_meta.replace(&*current_version, &*version);

    // Recreate the file and dump the processed contents to it
    let mut recreate_meta = try!(File::create(metadata));
    try!(recreate_meta.write_fmt(format_args!("{}", &new_metadata)));

    // Commit the changes made to the metadata
    let mut commit_msg = String::from("Bump version to ");
    commit_msg.push_str(&version);
    try!(git::git_command(&["add", metadata.to_str().unwrap()], &utils::cwd()));
    try!(git::git_command(&["commit", "-m", &commit_msg], &utils::cwd()));
    Ok(())
}

// @Public

// Bump the metadata version, only if:
// * The project is a cookbook
// * The version hasn't been updated
//
// @param p_root [&PathBuf] The project root path
// @param pipeline [&str] Pipeline the change is targeting to
// @return () if success
pub fn bump_version(p_root: &PathBuf, pipeline: &str) -> Result<(), DeliveryError> {
    if is_cookbook(&p_root) {
        let project = try!(project::project_from_cwd());
        say("white", "Project ");
        say("yellow", &project);
        sayln("white", " is a cookbook");
        sayln("white", "Validating version in metadata");

        let meta_f_p = PathBuf::from(metadata_file(&p_root));
        let current_v = try!(metadata_version_from(try!(read_file(&meta_f_p))));
        let mut t_file = pipeline.to_string();
        t_file.push_str(":metadata.rb");
        let pipeline_meta = try!(git::git_command(&["show", &t_file], &p_root));
        let pipeline_v = try!(metadata_version_from(pipeline_meta.stdout));

        if current_v == pipeline_v {
            say("yellow", "The version hasn't been updated (");
            say("red", &pipeline_v);
            sayln("yellow", ")");
            let new_version = try!(bump_patchset(pipeline_v));
            say("white", "Bumping version to: ");
            sayln("green", &new_version);
            try!(save_version(&meta_f_p, new_version));
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn verify_version_bumping_using_bump_patchset() {
        let version = String::from("1.2.3");
        assert_eq!(String::from("1.2.4"), super::bump_patchset(version).unwrap());
    }

    #[test]
    fn return_the_metadata_file_path() {
        let project_path = PathBuf::from("/cookbook");
        assert_eq!(String::from("/cookbook/metadata.rb"), super::metadata_file(&project_path));
    }
}
