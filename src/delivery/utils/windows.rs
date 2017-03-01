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

use std::process::Command;
use std::env;
use std::fs;
use std::io;
use errors::{DeliveryError, Kind};
use std::path::{Path, PathBuf};
use std::convert::AsRef;

pub fn copy_recursive<A, B>(f: &A, t: &B) -> Result<(), DeliveryError>
        where A: AsRef<Path> + ?Sized,
              B: AsRef<Path> + ?Sized {
    let from = f.as_ref();
    let to = t.as_ref();
    let result = try!(make_command("Copy-Item")
                      .arg("-recurse")
                      .arg("-Force")
                      .arg(from.to_str().unwrap())
                      .arg(to.to_str().unwrap())
                      .output());
    super::cmd_success_or_err(&result, Kind::CopyFailed)
}

pub fn remove_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    match fs::metadata(path) {
        Ok(_) => {
            // only remove if there is something there
            let result = try!(make_command("Remove-Item")
                              .arg("-recurse")
                              .arg("-force")
                              .arg(path.as_ref().to_str().unwrap())
                              .output());
            super::cmd_success_or_err(&result, Kind::RemoveFailed)
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // probably should get specific. Re-raise unless this is
            // not found
            Ok(())
        },
        Err(e) => {
            let detail = format!("remove_recursive of '{}' failed: {}",
                                 path.as_ref().to_str().unwrap(), e);
            Err(DeliveryError{ kind: Kind::RemoveFailed,
                               detail: Some(detail) })
        }
    }
}

pub fn make_command(cmd: &str) -> Command {
    // could do "cmd.exe /c cmd" instead and less overhead.
    let mut c = Command::new("powershell.exe");
    c.arg("-noprofile")
        .arg("-nologo")
        .arg("-command")
        .arg(cmd);
    c
}

/// Returns the absolute path for a given command, if it exists, by searching the `PATH`
/// environment variable.
///
/// If the command represents an absolute path, then the `PATH` seaching will not be performed.
/// If no absolute path can be found for the command, then `None` is returned.
///
/// On Windows, the PATHEXT environment variable contains common extensions for commands,
/// for example allowing "docker.exe" to be found when searching for "docker".
pub fn find_command(command: &str) -> Option<PathBuf> {
    // If the command path is absolute and a file exists, then use that.
    let candidate = PathBuf::from(command);
    if candidate.is_absolute() && candidate.is_file() {
        return Some(candidate);
    }
    // Find the command by checking each entry in `PATH`. If we still can't find it,
    // give up and return `None`.
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let candidate = PathBuf::from(&path).join(command);
            if candidate.is_file() {
                return Some(candidate)
            }
            if let Some(command) = find_command_with_pathext(&candidate) {
                return Some(command)
            }
        }
    }
    None
}

// Windows relies on path extensions to resolve commands like `docker` to `docker.exe`
// Path extensions are found in the PATHEXT environment variable.
// We should only search with PATHEXT if the file does not already have an extension.
fn find_command_with_pathext(candidate: &PathBuf) -> Option<PathBuf> {
    if candidate.extension().is_none() {
        if let Some(pathexts) = env::var_os("PATHEXT") {
            let pathexts = env::split_paths(&pathexts).filter_map(|e| {
                e.to_str().map(|s| String::from(s))
            });
            for pathext in pathexts {
                let candidate = candidate.with_extension(pathext.trim_matches('.'));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

pub fn stunnel_path() -> String {
    String::from("C:\\opscode\\chefdk\\embedded\\bin\\stunnel.exe")
}

pub fn chefdk_openssl_path() -> String {
    String::from("C:\\opscode\\chefdk\\embedded\\bin\\openssl.exe")
}

// ---------------
// dummy functions
// ---------------
//
// These functions are no-ops to allow for compatibility with unix
// system. For now, we aren't attempting to handle user/privilege
// dropping on Windows.
//
#[allow(unused_variables)]
pub fn chmod<P: ?Sized>(path: &P, setting: &str) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    Ok(())
}

#[allow(unused_variables)]
pub fn chown_all<P: AsRef<Path>>(who: &str,
                            paths: &[P]) ->  Result<(), DeliveryError>
{
    Ok(())
}

pub fn privileged_process() -> bool {
    true
}
// -------------------
// end dummy functions
// -------------------
