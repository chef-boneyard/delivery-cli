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

pub mod init;
pub mod review;
pub mod local;
pub mod setup;
pub mod token;
pub mod checkout;
pub mod diff;
pub mod clone;
pub mod api;
pub mod job;
pub mod status;

use std;
use utils;
use types::{DeliveryResult, ExitCode};

pub trait Command: Sized {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        let _ = child_processes;
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode>;

    fn teardown(&self, child_processes: Vec<std::process::Child>) -> DeliveryResult<()> {
        utils::kill_child_processes(child_processes)
    }
}
