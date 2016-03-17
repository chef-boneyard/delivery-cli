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
use regex::Captures;

#[derive(Debug, Clone)]
pub struct MetadataVersion {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl MetadataVersion {
    pub fn new(ma: Option<usize>, mi: Option<usize>, pa: Option<usize>) -> MetadataVersion {
        MetadataVersion {
            major: ma.unwrap_or_default(),
            minor: mi.unwrap_or_default(),
            patch: pa.unwrap_or_default()
        }
    }

    pub fn to_string(&self) -> String {
        [
            self.major.to_string(), ".".to_string(),
            self.minor.to_string(), ".".to_string(),
            self.patch.to_string()
        ].concat()
    }
}

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
        let meta_f_c = try!(read_file(&meta_f_p));
        let current_meta_v = try!(metadata_version_from(&meta_f_c));
        let current_v = current_meta_v.to_string();
        let mut t_file = pipeline.to_string();
        t_file.push_str(":metadata.rb");
        let pipeline_meta = try!(git::git_command(&["show", &t_file], &p_root));
        let pipeline_meta_v = try!(metadata_version_from(&pipeline_meta.stdout));
        let pipeline_v = pipeline_meta_v.to_string();

        if current_v == pipeline_v {
            say("yellow", "The version hasn't been updated (");
            say("red", &pipeline_v);
            sayln("yellow", ")");
            let new_meta_version = try!(bump_patchset(pipeline_meta_v.clone()));
            let new_version = new_meta_version.to_string();
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

// @Private

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
//
// There are two valid version formats:
// a) 'x.y.z' - The normal semantic version. (major.minor.patch)
// b) 'x.y'   - Where the patchset will be 0 by default. (major.minor.0)
fn metadata_version_from(content: &str) -> Result<MetadataVersion, DeliveryError> {
    for l in content.lines() {
        let r_m_m_p = Regex::new(r"version\s+'(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)'").unwrap();
        if let Some(version) = r_m_m_p.captures(l) {
            return generate_metadata_version(version);
        }
        let r_m_m = Regex::new(r"version\s+'(?P<major>\d+)\.(?P<minor>\d+)'").unwrap();
        if let Some(version) = r_m_m.captures(l) {
            return generate_metadata_version(version);
        }
    };
    return Err(DeliveryError{ kind: Kind::MissingMetadataVersion, detail: None })
}

fn generate_metadata_version(metadata: Captures) -> Result<MetadataVersion, DeliveryError> {
    let mut ma = None;
    let mut mi = None;
    let mut pa = None;
    if let Some(major) = metadata.name("major") {
        ma = major.parse::<usize>().ok();
    };
    if let Some(minor) =  metadata.name("minor") {
        mi = minor.parse::<usize>().ok();
    };
    if let Some(patch) = metadata.name("patch") {
        pa = patch.parse::<usize>().ok();
    };
    Ok(MetadataVersion::new(ma, mi, pa))
}

// Bump the patchset of the provided version
fn bump_patchset(mut version: MetadataVersion) -> Result<MetadataVersion, DeliveryError> {
    version = MetadataVersion { patch: version.patch + 1, .. version };
    Ok(version)
}

// Saves the new version to the metadata and commit the changes
fn save_version(metadata: &PathBuf, version: String) -> Result<(), DeliveryError> {
    let current_meta = try!(read_file(metadata));
    let current_meta_version = try!(metadata_version_from(&current_meta));
    let current_version = current_meta_version.to_string();
    let new_metadata = current_meta.replace(&*current_version, &*version);

    // Recreate the file and dump the processed contents to it
    let mut recreate_meta = try!(File::create(metadata));
    try!(recreate_meta.write(new_metadata.as_bytes()));

    // Commit the changes made to the metadata
    let mut commit_msg = String::from("Bump version to ");
    commit_msg.push_str(&version);
    try!(git::git_command(&["add", metadata.to_str().unwrap()], &utils::cwd()));
    try!(git::git_command(&["commit", "-m", &commit_msg], &utils::cwd()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use cookbook::*;

    #[test]
    fn test_metadata_version_constructor() {
        let version_generator = MetadataVersion::new(None, Some(2), None);
        let MetadataVersion { major: ma, minor: mi, patch: pa } = version_generator;
        assert_eq!(ma, 0);
        assert_eq!(mi, 2);
        assert_eq!(pa, 0);
        assert_eq!("0.2.0", &version_generator.to_string());
    }
    #[test]
    fn verify_version_bumping_using_bump_patchset() {
        let version = MetadataVersion { major: 1, minor: 2, patch: 3 };
        let MetadataVersion { major: ma, minor: mi, patch: pa } = super::bump_patchset(version).unwrap();
        assert_eq!(ma, 1);
        assert_eq!(mi, 2);
        assert_eq!(pa, 4);
    }

    #[test]
    fn return_the_metadata_file_path() {
        let project_path = PathBuf::from("/cookbook");
        assert_eq!(String::from("/cookbook/metadata.rb"), super::metadata_file(&project_path));
    }

    #[test]
    fn verify_happy_metadata_version_from_content() {
        let happy_version = "1.2.3";
        let happy_metadata_content = metadata_from_version(&happy_version);
        let happy_metadata_version = super::metadata_version_from(&happy_metadata_content).unwrap();
        assert_eq!(happy_version, &happy_metadata_version.to_string());

        let valid_version = "1.2";
        let valid_metadata_content = metadata_from_version(&valid_version);
        let valid_metadata_version = super::metadata_version_from(&valid_metadata_content).unwrap();
        assert_eq!("1.2.0", &valid_metadata_version.to_string());

        let awesome_version = "123.123.123";
        let awesome_metadata_content = metadata_from_version(&awesome_version);
        let awesome_metadata_version = super::metadata_version_from(&awesome_metadata_content).unwrap();
        assert_eq!(awesome_version, &awesome_metadata_version.to_string());
    }

    #[test]
    fn verify_sad_metadata_version_from_content() {
        let sad_version = "1..";
        let sad_metadata =  metadata_from_version(&sad_version);
        assert!(super::metadata_version_from(&sad_metadata).is_err());

        let typo_version = "1.2.";
        let typo_metadata =  metadata_from_version(&typo_version);
        assert!(super::metadata_version_from(&typo_metadata).is_err());

        let no_version = "";
        let no_metadata =  metadata_from_version(&no_version);
        assert!(super::metadata_version_from(&no_metadata).is_err());
    }

    // Quick helper to render a dummy metadata from a provided version
    fn metadata_from_version(version: &str) -> String {
        String::from(["version '", version, "'"].concat())
    }
}
