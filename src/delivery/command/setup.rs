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

use cli::setup::SetupClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::sayln;
use std::path::PathBuf;
use config::Config;
use command::Command;

pub struct SetupCommand<'n> {
    pub options: &'n SetupClapOptions<'n>,
    pub config: &'n Config,
    pub config_path: &'n PathBuf,
}

impl<'n> Command for SetupCommand<'n> {
    fn run(self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");
        try!(self.config.write_file(self.config_path));
        Ok(0)
    }
}
