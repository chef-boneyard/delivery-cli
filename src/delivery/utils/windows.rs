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
use std::fs;
use std::io;
use errors::{DeliveryError, Kind};
use std::path::Path;
use std::convert::AsRef;

pub fn copy_recursive<P: ?Sized>(f: &P, t: &P) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    let from = f.as_ref();
    let to = t.as_ref();
    let result = try!(make_command("Copy-Item")
                      .arg("-recurse")
                      .arg("-Force")
                      .arg(from.to_str().unwrap())
                      .arg(to.to_str().unwrap())
                      .output());
    if !result.status.success() {
        let detail = Some(format!("STDOUT: {}\nSTDERR: {}",
                                  String::from_utf8_lossy(&result.stdout),
                                  String::from_utf8_lossy(&result.stderr)));
        Err(DeliveryError{ kind: Kind::CopyFailed, detail: detail })
    } else {
        Ok(())
    }
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
            if !result.status.success() {
                let detail = Some(format!("STDOUT: {}\nSTDERR: {}",
                                          String::from_utf8_lossy(&result.stdout),
                                          String::from_utf8_lossy(&result.stderr)));
                Err(DeliveryError{ kind: Kind::RemoveFailed, detail: detail })
            } else {
                Ok(())
            }
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
