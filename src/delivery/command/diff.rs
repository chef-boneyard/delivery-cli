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
use cli::diff::DiffClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{say, sayln};

pub fn run(opts: DiffClapOptions) -> DeliveryResult<ExitCode> {
    sayln("green", "Chef Delivery");
    let config = try!(cli::init_command(&opts));
    let target = validate!(config, pipeline);
    say("white", "Showing diff for ");
    say("yellow", opts.change);
    say("white", " targeted for pipeline ");
    say("magenta", &target);

    if opts.patchset == "latest" {
        sayln("white", " latest patchset");
    } else {
        say("white", " at patchset ");
        sayln("yellow", opts.patchset);
    }
    try!(git::diff(opts.change, opts.patchset, &target, &opts.local));
    Ok(0)
}
