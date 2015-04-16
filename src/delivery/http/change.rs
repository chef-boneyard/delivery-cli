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

use token::TokenStore;

#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, PartialOrd)]
pub struct Description {
    pub title: String,
    pub description: String
}

impl Description {
    pub fn payload(title: &str, desc: &str) -> Result<String, DeliveryError> {
        let desc = Description{ title: String::from_str(title),
                                description: String::from_str(desc) };
        desc.to_json()
    }

    pub fn to_json(&self) -> Result<String, DeliveryError> {
        let payload = try!(json::encode(&self));
        Ok(payload)
    }

    pub fn parse_json(response: &str) -> Result<Description, DeliveryError> {
        let description: Description = try!(json::decode(response));
        Ok(description)
    }

    pub fn parse_text(text: &str) -> Result<Description, DeliveryError> {
        let mut items: Vec<&str> = text.lines_any().collect();
        let title = items[0].to_string();
        let desc = if items.len() > 1 {
            items.remove(0);
            items.connect("\n").trim().to_string()
        } else {
            "".to_string()
        };
        Ok(Description{ title: title, description: desc })
    }
}

/// Fetch the description for a change
pub fn get(server: &str, ent: &str, user: &str, org: &str,
           proj: &str, change: &str) -> Result<Description, DeliveryError> {
    let mut client = APIClient::new_https(&server, &ent);
    let tstore = try!(TokenStore::from_home());
    let auth = try!(APIAuth::from_token_store(tstore, &server, &ent, &user));
    client.set_auth(auth);
    let path = format!("orgs/{}/projects/{}/changes/{}/description",
                       org, proj, change);
    debug!("description path: {}", path);
    let mut result = try!(client.get(&path));
    match result.status {
        StatusCode::Ok => {
            let mut body_string = String::new();
            let _x = try!(result.read_to_string(&mut body_string));
            let description = try!(Description::parse_json(&body_string));
            Ok(description)
        },
        StatusCode::Unauthorized => {
            let msg = "API request returned 401".to_string();
            Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                               detail: Some(msg)})
        },
        error_code @ _ => {
            let msg = format!("API request returned {}",
                              error_code);
            Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                               detail: Some(msg)})
        }
    }
}

/// Set the description for a change
pub fn set(server: &str, ent: &str, user: &str, org: &str, proj: &str,
           change: &str,
           description: &Description) -> Result<(), DeliveryError> {
    let mut client = APIClient::new_https(&server, &ent);
    let tstore = try!(TokenStore::from_home());
    let auth = try!(APIAuth::from_token_store(tstore, &server, &ent, &user));
    client.set_auth(auth);
    let path = format!("orgs/{}/projects/{}/changes/{}/description",
                       org, proj, change);
    let payload = try!(description.to_json());
    let result = try!(client.put(&path, &payload));
    match result.status {
        StatusCode::NoContent => Ok(()),
        StatusCode::Unauthorized => {
            let msg = "API request returned 401".to_string();
            Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                               detail: Some(msg)})
        },
        error_code @ _ => {
            let msg = format!("API request returned {}",
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
    fn description_payload_test() {
        let payload = Description::payload("a title", "so descriptive!");
        let expect = "{\"title\":\"a title\",\"description\":\"so descriptive!\"}";
        assert_eq!(expect, payload.unwrap());
    }

    #[test]
    fn description_to_json_test() {
        let desc = Description { title: "a title".to_string(),
                                 description: "so descriptive!".to_string() };
        let payload = desc.to_json().unwrap();
        let expect = "{\"title\":\"a title\",\"description\":\"so descriptive!\"}";
        assert_eq!(expect, payload);
    }

    #[test]
    fn description_parse_json_test() {
        let response = "{\"title\":\"a title\",\"description\":\"so descriptive!\"}";
        let expect = Description{ title: "a title".to_string(),
                                  description: "so descriptive!".to_string()};
        let description = Description::parse_json(response).unwrap();
        assert_eq!(expect, description);
    }

    #[test]
    fn description_parse_text_1_test() {
        let text = "Just a title";
        let expect = Description{ title: text.to_string(),
                                  description: "".to_string() };
        let desc = Description::parse_text(text).unwrap();
        assert_eq!(expect, desc);
    }

    #[test]
    fn description_parse_text_2_test() {
        let text = "Just a title\n\nWith some description";
        let expect = Description{ title: "Just a title".to_string(),
                                  description: "With some description".to_string() };
        let desc = Description::parse_text(text).unwrap();
        assert_eq!(expect, desc);
    }

    #[test]
    fn description_parse_text_3_test() {
        let text = "Just a title\n\nL1\nL2\nL3\n";
        let expect = Description{ title: "Just a title".to_string(),
                                  description: "L1\nL2\nL3".to_string() };
        let desc = Description::parse_text(text).unwrap();
        assert_eq!(expect, desc);
    }

}
