//
// Copyright:: Copyright (c) 2017 Chef Software, Inc.
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
use http::APIClient;
use hyper::status::StatusCode;

impl APIClient {
    // Verify if the provided user exists
    pub fn user_exists(&self, user: &str) -> bool {
        let path = format!("users/{}", user);
        self.get(&path).and_then(|response| {
            if let StatusCode::Ok = response.status {
                return Ok(true)
            }
            Ok(false)
        }).unwrap_or(false)
    }
}
