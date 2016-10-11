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
use cli::token::TokenClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{turn_on_output, turn_off_output, sayln};
use utils::cwd;
use token::TokenStore;

pub fn run(opts: TokenClapOptions) -> DeliveryResult<ExitCode> {
    // If we want the raw token, we wont print any output
    // during the token request so we will disable it
    if opts.raw {
        turn_off_output();
    }

    sayln("green", "Chef Delivery");
    let mut config = try!(cli::load_config(&cwd()));
    config = config.set_server(opts.server)
        .set_api_port(opts.port)
        .set_enterprise(opts.ent)
        .set_user(opts.user);

    if opts.saml.is_some() {
        config.saml = opts.saml;
    }

    let token: String = if opts.verify {
        try!(TokenStore::verify_token(&config))
    } else {
        try!(TokenStore::request_token(&config))
    };

    if opts.raw {
        turn_on_output();
        sayln("white", &format!("{}", &token));
    }

    Ok(0)
}
