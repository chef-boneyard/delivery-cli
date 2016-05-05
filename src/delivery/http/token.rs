//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
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
pub struct TokenRequest {
    username: String,
    password: String
}

impl TokenRequest {
    pub fn payload(user: &str, pass: &str) -> Result<String, DeliveryError> {
        let treq = TokenRequest{  username: String::from(user),
                                  password: String::from(pass) };
        let payload = try!(json::encode(&treq));
        Ok(payload)
    }
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct TokenResponse {
    token: String
}

impl TokenResponse {
    pub fn parse_token(response: &str) -> Result<String, DeliveryError> {
        let tresponse: TokenResponse = try!(json::decode(response));
        Ok(tresponse.token)
    }
}

pub fn verify(config: &Config) -> Result<bool, DeliveryError> {
    let client = try!(APIClient::from_config(config));
    let mut result = try!(client.get("orgs"));
    match result.status {
        StatusCode::Ok => Ok(true),
        StatusCode::Unauthorized => {
            let detail = try!(APIClient::extract_pretty_json(&mut result));
            match detail.find("token_expired") {
                Some(_) => Ok(false),
                None => {
                    let mut msg = "API request returned 401\nDetails:\n".to_string();
                    msg.push_str(&detail);
                    Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                                       detail: Some(msg)})
                }
            }
        },
        _ => {
            let pretty_json = try!(APIClient::extract_pretty_json(&mut result));
            println!("{}", pretty_json);
            Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                               detail: Some(pretty_json)})
        }
    }
}

/// Request an API token for a user from a Delivery server.
pub fn request(config: &Config, pass: &str) -> Result<String, DeliveryError> {
    let client = try!(APIClient::from_config_no_auth(config));
    let user = try!(config.user());
    let payload = try!(TokenRequest::payload(&user, pass));
    let path = format!("users/{}/get-token", &user);
    let mut result = try!(client.post(&path, &payload));
    match result.status {
        StatusCode::Ok => {
            let mut body_string = String::new();
            let _x = try!(result.read_to_string(&mut body_string));
            let token = try!(TokenResponse::parse_token(&body_string));
            Ok(token)
        },
        StatusCode::Unauthorized => {
            let ent = try!(config.enterprise());
            let server = try!(config.server());
            let msg = format!("Details: server={}, enterprise={}, user={}",
                              &server, &ent, &user);
            Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                               detail: Some(msg)})
        },
        error_code @ _ => {
            let msg = format!("token request returned {}",
                              error_code);
            Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                               detail: Some(msg)})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_request_payload_test() {
        let payload = TokenRequest::payload("alice", "sesame123");
        let expect = "{\"username\":\"alice\",\"password\":\"sesame123\"}";
        assert_eq!(expect, payload.unwrap());
    }

    #[test]
    fn token_response_parse_token_test() {
        let response = "{\"token\":\"abc123\"}";
        let token = TokenResponse::parse_token(response).unwrap();
        assert_eq!("abc123", token);
    }
}
