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

use errors::{DeliveryError, Kind};
use std::convert::AsRef;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use utils::path_join_many::PathJoinMany;

pub mod say;
pub mod path_join_many;
pub mod open;


#[cfg(not(target_os = "windows"))]
pub use self::unix::*;

#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

pub fn mkdir_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError> where P: AsRef<Path> {
    try!(fs::create_dir_all(path.as_ref()));
    Ok(())
}

pub fn home_dir(to_append: &str) -> Result<PathBuf, DeliveryError>
{
   match env::home_dir() {
       Some(home) => Ok(home.join_many(&[to_append])),
       None => {
           let msg = "unable to find home dir".to_string();
           return Err(DeliveryError{ kind: Kind::NoHomedir,
                                     detail: Some(msg) })
       }
   }
}
