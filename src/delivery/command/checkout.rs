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
use cli::checkout::CheckoutClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{sayln, say};
use utils::cwd;

pub fn run(opts: CheckoutClapOptions) -> DeliveryResult<ExitCode> {
    sayln("green", "Chef Delivery");
    let mut config = try!(cli::load_config(&cwd()));
    config = config.set_pipeline(opts.pipeline);
    let target = validate!(config, pipeline);
    say("white", "Checking out ");
    say("yellow", opts.change);
    say("white", " targeted for pipeline ");
    say("magenta", &target);

    let pset = match opts.patchset {
        "" | "latest" => {
            sayln("white", " tracking latest changes");
            "latest"
        },
        p @ _ => {
            say("white", " at patchset ");
            sayln("yellow", p);
            p
        }
    };
    try!(git::checkout_review(opts.change, pset, &target));
    Ok(0)
}
