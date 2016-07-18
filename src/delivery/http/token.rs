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
use token::TokenStore;
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

    pub fn parse_token_expired(content: &str) -> bool {
        match content.find("token_expired") {
            Some(_) => true,
            None => false
        }
    }
}

// Verify an API token for a user against a Delivery Server
//
// This method verifies that a user has an existing Token on disk,
// that it is valid and has not yet expired. Otherwise it will return
// false saying that a token needs to be regenerated
pub fn verify(config: &Config) -> Result<bool, DeliveryError> {
    let api_server = try!(config.api_host_and_port());
    let ent = try!(config.enterprise());
    let user = try!(config.user());
    let tstore = try!(TokenStore::from_home());
    let auth = try!(APIAuth::from_token_store(tstore, &api_server, &ent, &user).or_else(|e| {
        debug!("Ignoring {:?}\nRequesting token from config", e);
        APIAuth::from_token_request(&config)
    }));
    let client = try!(APIClient::from_config_no_auth(config).and_then((|mut c| {
        c.set_auth(auth);
        Ok(c)
    })));
    let mut response = try!(client.get("orgs"));
    match response.status {
        StatusCode::Ok => Ok(true),
        StatusCode::Unauthorized => {
            let content = try!(APIClient::extract_pretty_json(&mut response));
            // Send verify(false) if the token has expired
            Ok(!TokenResponse::parse_token_expired(&content))
        },
        _ => {
            let pretty_json = try!(APIClient::extract_pretty_json(&mut response));
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
            try!(result.read_to_string(&mut body_string));
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

    #[test]
    fn token_response_parse_token_expired_test() {
        let r_token_expired = "{\"error\":\"token_expired\"}";
        assert!(TokenResponse::parse_token_expired(r_token_expired));

        let r_token_denied = "{\"error\":\"token_denied\"}";
        assert!(!TokenResponse::parse_token_expired(r_token_denied));

        let r_other = "{\"orgs\":\"[]\"}";
        assert!(!TokenResponse::parse_token_expired(r_other))
    }
}
