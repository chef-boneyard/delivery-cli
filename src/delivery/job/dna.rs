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

use job::change::{BuilderCompat, Change};
use serde_json::Value as SerdeJson;

#[derive(Serialize)]
pub struct Top {
    pub workspace_path: String,
    pub workspace: WorkspaceCompat,
    pub change: Change,
    // Use a generic Json format
    //
    // There are projects that have custom attributes inside the
    // `.delivery/config.json` that needs to be available on every
    // phase of the pipeline so that the build_cookbook can use them.
    //
    // A clear example of this is in the `delivery-truck` build_cookbook:
    // => https://github.com/chef-cookbooks/delivery-truck/blob/master/.delivery/config.json#L10-L22
    //
    // For this reason we need to use a generic Json format until we
    // have a reserved word in the config that allow us to be prescriptive
    // about the content of the file that can be configurable.
    //
    // TODO: Restrict the config.json format by selecting a reserved field
    // that allow users to inject attributes that will be used by the
    // build_cookbook. (for example a field called `attributes`)
    pub config: SerdeJson,
}

#[derive(Serialize)]
pub struct DNA {
    pub delivery: Top,
    pub delivery_builder: BuilderCompat,
}

#[derive(Serialize)]
pub struct WorkspaceCompat {
    pub root: String,
    pub chef: String,
    pub cache: String,
    pub repo: String,
    pub ssh_wrapper: String,
}
