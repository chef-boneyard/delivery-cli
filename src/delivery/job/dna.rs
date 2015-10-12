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

use rustc_serialize::json;
use job::change::{Change, BuilderCompat};

#[derive(RustcEncodable)]
pub struct Top {
    pub workspace_path: String,
    pub workspace: WorkspaceCompat,
    pub change: Change,
    pub config: json::Json
}

#[derive(RustcEncodable)]
pub struct DNA {
    pub delivery: Top,
    pub delivery_builder: BuilderCompat
}

#[derive(RustcEncodable)]
pub struct WorkspaceCompat {
    pub root: String,
    pub chef: String,
    pub cache: String,
    pub repo: String,
    pub ssh_wrapper: String
}
