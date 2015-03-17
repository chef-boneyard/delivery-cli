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

use std::path::{Path, PathBuf};

/// You were too useful to die, join_many. This implements
/// what used to be the join_many method on old_path. Feed
/// it an array of &str's, and it will push them on to a
/// PathBuf, then return the final PathBuf.
pub trait PathJoinMany {
    fn join_many(&self, paths: &[&str]) -> PathBuf;
}

impl PathJoinMany for PathBuf {
    fn join_many(&self, paths: &[&str]) -> PathBuf {
        let mut buf = self.clone();
        for p in paths {
            buf = buf.join(p);
        }
        buf
    }
}

impl PathJoinMany for Path {
    fn join_many(&self, paths: &[&str]) -> PathBuf {
        let mut buf = self.to_path_buf();
        for p in paths {
            buf = buf.join(p);
        }
        buf
    }
}
