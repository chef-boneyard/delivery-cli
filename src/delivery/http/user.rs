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
    saml_user: bool
}

impl LookupResponse {
    pub fn parse_saml_user(response: &str) -> Result<bool, DeliveryError> {
        let lresponse: LookupResponse = try!(json::decode(response));
        Ok(lresponse.saml_user)
    }
}

/// Lookup if user is a SAML-backed user on a Delivery server.
pub fn is_saml(config: &Config) -> Result<bool, DeliveryError> {
    let client = try!(APIClient::from_config_no_auth(config));
    let user = try!(config.user());
    let path = format!("saml/lookup-user/{}", &user);
    let mut result = try!(client.get(&path));
    match result.status {
        StatusCode::Ok => {
            let mut body_string = String::new();
            try!(result.read_to_string(&mut body_string));
            let resp = try!(LookupResponse::parse_saml_user(&body_string));
            Ok(resp)
        },
        error_code @ _ => {
            let msg = format!("SAML lookup request returned {}",
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
        let response = "{\"saml_user\":true}";
        let saml = LookupResponse::parse_saml_user(response).unwrap();
        assert_eq!(true, saml);

        let response = "{\"saml_user\":false}";
        let saml = LookupResponse::parse_saml_user(response).unwrap();
        assert_eq!(false, saml);
    }
}
