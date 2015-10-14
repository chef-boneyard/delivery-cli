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

//! Utility functions for checking metadata on files. Has a similar API to
//! the unstable `path_ext` module.

use std::fs;
use std::path::Path;

/// Return true if the given path is a file; otherwise false.
///
/// Will also return false if the file doesn't exist, or if the user doesn't
/// have permission to see the file.
pub fn is_file<P: ?Sized>(path: &P) -> bool where P: AsRef<Path> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        // We either don't exist, or we don't have permission to even see the file
        Err(_e) => return false
    };
    meta.is_file()
}

/// Return true if the given path is a directory; otherwise false.
///
/// Will also return false if the path doesn't exist, or if the user doesn't
/// have permission to see the path.
pub fn is_dir<P: ?Sized>(path: &P) -> bool where P: AsRef<Path> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        // We either don't exist, or we don't have permission to even see the file
        Err(_e) => return false
    };
    meta.is_dir()
}
