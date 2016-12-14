//
// Copyright:: Copyright (c) 2016 Chef Software, Inc.
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

use cli;
use git;
use cli::clone::CloneClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{say, sayln};
use utils::cwd;

pub fn run(opts: CloneClapOptions) -> DeliveryResult<ExitCode> {
    sayln("green", "Chef Delivery");
    let config = try!(cli::init_command(&opts));
    say("white", "Cloning ");
    let delivery_url = try!(config.delivery_git_ssh_url());
    let clone_url = if opts.git_url.is_empty() {
        delivery_url.clone()
    } else {
        String::from(opts.git_url)
    };
    say("yellow", &clone_url);
    say("white", " to ");
    sayln("magenta", &format!("{}", opts.project));
    try!(git::clone(opts.project, &clone_url));
    let project_root = cwd().join(opts.project);
    try!(git::create_or_update_delivery_remote(&delivery_url,
                                               &project_root));
    Ok(0)
}

