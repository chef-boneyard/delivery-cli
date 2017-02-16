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

use cli::token::TokenClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{turn_on_output, turn_off_output, sayln};
use token::TokenStore;
use config::Config;
use command::Command;

pub struct TokenCommand<'n> {
    pub options: &'n TokenClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for TokenCommand<'n> {
    fn run(self) -> DeliveryResult<ExitCode> {

        // If we want the raw token, we wont print any output
        // during the token request so we will disable it
        if self.options.raw {
            turn_off_output();
        }

        sayln("green", "Chef Delivery");

        let token: String = if self.options.verify {
            try!(TokenStore::verify_token(&self.config))
        } else {
            try!(TokenStore::request_token(&self.config))
        };

        if self.options.raw {
            turn_on_output();
            sayln("white", &format!("{}", &token));
        }

        Ok(0)
    }
}
