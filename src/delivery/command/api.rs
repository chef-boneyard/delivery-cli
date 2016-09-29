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
use config::Config;
use utils::cwd;
use http::APIClient;
use hyper::status::StatusCode;

pub fn run(opts: ApiClapOptions) -> DeliveryResult<ExitCode> {
    let mut config = try!(Config::load_config(&cwd()));
    config = config.set_user(opts.user)
        .set_server(opts.server)
        .set_api_port(opts.api_port)
        .set_enterprise(opts.ent);
    let client = try!(APIClient::from_config(&config));
    let mut result = match opts.method {
        "get" => try!(client.get(opts.path)),
        "post" => try!(client.post(opts.path, opts.data)),
        "put" => try!(client.put(opts.path, opts.data)),
        "delete" => try!(client.delete(opts.path)),
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
