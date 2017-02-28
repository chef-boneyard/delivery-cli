//
// Copyright:: Copyright (c) 2017 Chef Software, Inc.
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
// Takes in fully parsed and defaulted init clap args,
// executes init codeflow, handles user actionable errors, as well as UI output.
//
// Returns an integer exit code, handling all errors it knows how to
// and panicing on unexpected errors.
//
// mod test_paths
//
// The main purpose of this module is to provide methods to access
// the paths that the tests/ directory has inside. The module won't
// be loaded unless we are running the tests suits with `cargo test`
use std::env;
use std::path::PathBuf;
use super::path_join_many::PathJoinMany;

// Return the path of the generated delivery-cli executable
// => delivery-cli/target/debug/deps/delivery-*
pub fn exe_path() -> PathBuf {
    env::current_exe().unwrap()
}

// Return the root path of the tests folder:
// => delivery-cli/tests
pub fn root() -> PathBuf {
    let mut exe = exe_path(); // delivery-*/
    exe.pop();                // deps/
    exe.pop();                // debug/
    exe.pop();                // target/
    exe.pop();                // delivery-cli/
    exe.join("tests")         // tests/
}

// Return the fixtures path inside the tests folder:
// => delivery-cli/tests/fixtures
pub fn fixtures() -> PathBuf {
    root().join_many(&["fixtures"])
}

pub fn fixture_file(names: &str) -> PathBuf {
    fixtures().join_many(&[names])
}

pub fn join_many(v: &Vec<&str>) -> PathBuf {
    root().join_many(v)
}

