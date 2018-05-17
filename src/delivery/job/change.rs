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

#[derive(Serialize, Debug)]
pub struct Change {
    pub enterprise: String,
    pub organization: String,
    pub project: String,
    pub pipeline: String,
    pub change_id: String,
    pub patchset_number: String,
    pub stage: String,
    pub phase: String,
    pub git_url: String,
    pub sha: String,
    pub patchset_branch: String,
}

#[derive(Serialize, Debug)]
pub struct BuilderCompat {
    pub workspace: String,
    pub repo: String,
    pub cache: String,
    pub build_id: String,
    pub build_user: String,
}
