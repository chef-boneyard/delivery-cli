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

use cli::api::ApiClapOptions;
use types::{DeliveryResult, ExitCode};
use errors::{DeliveryError, Kind};
use http::APIClient;
use hyper::status::StatusCode;
use command::Command;
use config::Config;

pub struct ApiCommand<'n> {
    pub options: &'n ApiClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for ApiCommand<'n> {
    fn run(self) -> DeliveryResult<ExitCode> {
        let client = try!(APIClient::from_config(&self.config));
        let mut result = match self.options.method {
            "get" => try!(client.get(self.options.path)),
            "post" => try!(client.post(self.options.path, self.options.data)),
            "put" => try!(client.put(self.options.path, self.options.data)),
            "delete" => try!(client.delete(self.options.path)),
            _ => return Err(DeliveryError{ kind: Kind::UnsupportedHttpMethod,
                                           detail: None })
        };
        match result.status {
            StatusCode::NoContent => {},
            StatusCode::InternalServerError => {
                return Err(DeliveryError{ kind: Kind::InternalServerError, detail: None})
            },
            _ => {
                let pretty_json = try!(APIClient::extract_pretty_json(&mut result));
                println!("{}", pretty_json);
            }
        };
        Ok(0)
    }
}
