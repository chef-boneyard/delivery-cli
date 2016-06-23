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
use errors::{DeliveryError, Kind};
use libc;
use utils::path_to_string;
use std::path::Path;
use std::convert::AsRef;
use std::error;

pub fn copy_recursive<P: ?Sized>(f: &P, t: &P) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    let from = f.as_ref();
    let to = t.as_ref();
    let result = try!(Command::new("cp")
         .arg("-R")
         .arg("-a")
         .arg(from.to_str().unwrap())
         .arg(to.to_str().unwrap())
         .output());
    if !result.status.success() {
        Err(DeliveryError{
            kind: Kind::CopyFailed,
            detail: Some(format!("STDOUT: {}\nSTDERR: {}",
                                 String::from_utf8_lossy(&result.stdout),
                                 String::from_utf8_lossy(&result.stderr)))
        })
    } else {
        Ok(())
    }
}

pub fn remove_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    try!(Command::new("rm")
         .arg("-rf")
         .arg(path.as_ref().to_str().unwrap())
         .output());
    Ok(())
}

pub fn chmod<P: ?Sized>(path: &P, setting: &str) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    let result = try!(Command::new("chmod")
         .arg(setting)
         .arg(path.as_ref().to_str().unwrap())
         .output());
    if !result.status.success() {
        Err(DeliveryError{
            kind: Kind::ChmodFailed,
            detail: Some(format!("STDOUT: {}\nSTDERR: {}",
                                 String::from_utf8_lossy(&result.stdout),
                                 String::from_utf8_lossy(&result.stderr)))
        })
    } else {
        Ok(())
    }
}

pub fn chown_all<P: AsRef<Path>>(who: &str,
                                 paths: &[P]) -> Result<(), DeliveryError> {
    let mut command = Command::new("chown");
    command.arg("-R").arg(who);
    for p in paths {
        command.arg(&path_to_string(p));
    }
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => {
            return Err(DeliveryError{
                kind: Kind::FailedToExecute,
                detail: Some(format!("failed to execute chown: {}",
                                     error::Error::description(&e)))}) },
    };
    if !output.status.success() {
        Err(DeliveryError{
            kind: Kind::ChownFailed,
            detail: Some(format!("STDOUT: {}\nSTDERR: {}\n",
                                 String::from_utf8_lossy(&output.stdout),
                                 String::from_utf8_lossy(&output.stderr)))})
    } else {
        Ok(())
    }
}

pub fn privileged_process() -> bool {
    match unsafe { libc::getuid() } {
        0 => true,
        _ => false
    }
}

// Abstraction for command creation. Needed because of how we're
// wrapping commands in Windows. See this function in the
// corresponding windows module.
pub fn make_command(cmd: &str) -> Command {
    Command::new(cmd)
}

