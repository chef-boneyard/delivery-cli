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

/// Use this library to open a path or URL using the program
/// configured on the system.
///
/// # Usage
///
/// ```ignore
/// if utils::open::item("https://google.com").is_ok() {
///     println!("Look at your browser!");
/// }
/// ```
///
/// # Notes
/// As an operating system program is used, the open can fail.
/// Therfore, you are advised to at least check the result with
/// .is_err() and behave accordingly, e.g. by letting the user know
/// what you tried to open, and failed.
///
/// The following programs are used to attempt to open the item by
/// operating system:
///
/// * Linux: xdg-open, gnome-open, kde-open
/// * OS X: open
/// * Windows: start
///
use std::process::{Command, Output};
use errors::{DeliveryError, Kind};

// The MIT License (MIT)
// =====================

// Copyright © `2015` `Sebastian Thiel`

// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the “Software”), to deal in the Software without
// restriction, including without limitation the rights to use,
// copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.

#[cfg(target_os = "macos")]
pub fn item(path: &str) -> Result<(), DeliveryError> {
    item_for_cmds(path, &["open"])
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn item(path: &str) -> Result<(), DeliveryError> {
    item_for_cmds(path, &["xdg-open", "gnome-open", "kde-open"])
}

#[cfg(target_os = "windows")]
pub fn item(path: &str) -> Result<(), DeliveryError> {
    process_response("start", try!(Command::new("cmd.exe")
                                    .arg("/c")
                                    .arg("start")
                                    .arg(path)
                                    .output()))
}

#[cfg(not(target_os = "windows"))]
fn item_for_cmds(path: &str, cmds: &[&str]) -> Result<(), DeliveryError> {
    let mut res = Err(DeliveryError { kind: Kind::OpenFailed,
                                      detail: None});
    for cmd in cmds {
        res = item_for_cmd(path, cmd);
        match res {
            Ok(_) => break,
            Err(_) => continue,
        }
    }
    res
}

#[cfg(not(target_os = "windows"))]
fn item_for_cmd(path: &str, cmd: &str) -> Result<(), DeliveryError> {
    process_response(cmd, try!(Command::new(cmd).arg(path).output()))
}

fn process_response(cmd: &str, res: Output) -> Result<(), DeliveryError> {
    if res.status.success() {
        Ok(())
    } else {
        let code = match res.status.code() {
            Some(c) => format!("{}", c),
            None => format!("{}", "terminated by signal")
        };
        let msg = format!("Command '{}' failed with code {}",
                          cmd, code);
        Err(DeliveryError { kind: Kind::OpenFailed,
                            detail: Some(msg) })
    }
}
