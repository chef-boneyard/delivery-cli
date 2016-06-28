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

use errors::{DeliveryError, Kind};
use http::*;
use hyper::status::StatusCode;
use rustc_serialize::json;
use std::io::prelude::*;
use config::Config;

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct LookupResponse {
    enabled: bool
}

impl LookupResponse {
    pub fn parse_saml_enabled(response: &str) -> Result<bool, DeliveryError> {
        let lresponse: LookupResponse = try!(json::decode(response));
        Ok(lresponse.enabled)
    }
}

/// Lookup if Delivery server is SAML-enabled.
pub fn is_enabled(config: &Config) -> Result<bool, DeliveryError> {
    let client = try!(APIClient::from_config_no_auth(config));
    let path = "saml/enabled";
    let mut result = try!(client.get(&path));
    match result.status {
        StatusCode::Ok => {
            let mut body_string = String::new();
            try!(result.read_to_string(&mut body_string));
            let resp = try!(LookupResponse::parse_saml_enabled(&body_string));
            Ok(resp)
        },
        StatusCode::NotFound => { // 404 received if API does not exist
            debug!("endpoint 'saml/enabled' not found");
            Ok(false)
        },
        error_code @ _ => {
            let msg = format!("lookup of SAML authentication returned {}",
                              error_code);
            let mut detail = String::new();
            let e = match result.read_to_string(&mut detail) {
                Ok(_) => Ok(detail),
                Err(e) => Err(e)
            };
            Err(DeliveryError{ kind: Kind::ApiError(error_code, e),
                               detail: Some(msg)})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_response_parse_test() {
        let response = "{\"enabled\":true}";
        let saml = LookupResponse::parse_saml_enabled(response).unwrap();
        assert_eq!(true, saml);

        let response = "{\"enabled\":false}";
        let saml = LookupResponse::parse_saml_enabled(response).unwrap();
        assert_eq!(false, saml);
    }
}
