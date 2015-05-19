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

use std::fmt;
use std::env;
use hyper;
use hyper::status::StatusCode;
use hyper::client::response::Response as HyperResponse;
use hyper::HttpError;
use mime;
use rustc_serialize::json;
use errors::Kind as DelivError;
use std::error;
use std::io::prelude::*;
use errors::{DeliveryError, Kind};
use token::TokenStore;
use utils::say::{sayln};

mod headers;
pub mod token;
pub mod change;

#[derive(Debug)]
enum HProto {
    HTTP,
    HTTPS
}

impl fmt::Display for HProto {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &HProto::HTTP => "http",
            &HProto::HTTPS => "https"
        };
        write!(f, "{}", s)
    }
}

// impl string::ToString for HProto {
//     fn fmt(&self) -> String {
//         let s = match self {
//             &HProto::HTTP => "http",
//             &HProto::HTTPS => "https"
//         };
//         String::new(s)
//     }
// }

#[derive(Debug)]
pub struct APIClient {
    proto: HProto,
    host: String,
    enterprise: String,
    auth: Option<APIAuth>
}

impl APIClient {

    /// Create a new `APIClient` using HTTP attached to the enterprise
    /// given by `ent`.
    pub fn new_http(host: &str, ent: &str) -> APIClient {
        APIClient::new(HProto::HTTP, host, ent)
    }

    /// Create a new `APIClient` using HTTPS attached to the
    /// enterprise given by `ent`.
    pub fn new_https(host: &str, ent: &str) -> APIClient {
        APIClient::new(HProto::HTTPS, host, ent)
    }

    fn new(proto: HProto, host: &str, ent: &str) -> APIClient {
        APIClient {
            proto: proto,
            host: String::from_str(host),
            enterprise: String::from_str(ent),
            auth: None
        }
    }

    pub fn get_auth_from_home(&mut self, server: &str, ent: &str, user: &str) -> Result<APIAuth, DeliveryError> {
        match TokenStore::from_home() {
            Ok(tstore) => {
                APIAuth::from_token_store(tstore, server, ent, user)
            },
            Err(e) => Err(e)
        }
    }

    pub fn set_auth(&mut self, auth: APIAuth) {
        self.auth = Some(auth);
    }
    
    pub fn api_url(&self, path: &str) -> String {
        format!("{}://{}/api/v0/e/{}/{}",
                self.proto, self.host, self.enterprise, path)
    }

    pub fn project_exists(&self,
                          org: &str,
                          proj: &str) -> bool {

        let path = format!("orgs/{}/projects/{}", org, proj);
        match self.get(&path) {
            Ok(res) => {
                match res.status {
                    StatusCode::Ok => {
                        return true;
                    },
                    _ => {
                        return false;
                    }
                }
            },
            Err(e) => {
                sayln("red", &format!("project_exists: HttpError: {:?}", e));
                return false;
            }
        }
    }

    pub fn create_project(&self,
                          org: &str,
                          proj: &str) -> Result<HyperResponse, DelivError> {
        let path = format!("orgs/{}/projects", org);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let payload = format!("{{\"name\":\"{}\"}}", proj);
        match self.post(&path, &payload) {
            Ok(mut res) => {
                match res.status {
                    StatusCode::Created =>
                        Ok(res),
                    _ => {
                        let mut detail = String::new();
                        let e = match res.read_to_string(&mut detail) {
                            Ok(_) => Ok(detail),
                            Err(e) => Err(e)
                        };
                        Err(DelivError::ApiError(res.status, e))
                    }
                }
            },
            Err(e) => Err(DelivError::HttpError(e))
        }
    }

    pub fn create_pipeline(&self,
                           org: &str,
                           proj: &str,
                           pipe: &str) -> Result<HyperResponse, DelivError> {
        let path = format!("orgs/{}/projects/{}/pipelines", org, proj);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let base_branch = "master";
        let payload = format!("{{\"name\":\"{}\",\"base\":\"{}\"}}",
                              pipe, base_branch);
        match self.post(&path, &payload) {
            Ok(mut res) => {
                match res.status {
                    StatusCode::Created =>
                        Ok(res),
                    _ => {
                        let mut detail = String::new();
                        let e = match res.read_to_string(&mut detail) {
                            Ok(_) => Ok(detail),
                            Err(e) => Err(e)
                        };
                        Err(DelivError::ApiError(res.status, e))
                    }
                }
            },
            Err(e) => Err(DelivError::HttpError(e))
        }
    }

    pub fn get(&self, path: &str) -> Result<HyperResponse, HttpError> {
        let url = self.api_url(path);
        let mut client = hyper::Client::new();
        let req = client.get(&url);
        let req = req.header(self.json_content());
        let req = match self.auth {
            Some(ref auth) => {
                let (deliv_user, deliv_token) = auth.auth_headers();
                req.header(deliv_user).header(deliv_token)
            },
            None => req
        };
        req.send()
    }

    pub fn post(&self,
                path: &str,
                payload: &str) -> Result<HyperResponse, HttpError> {
        let url = self.api_url(path);
        let mut client = hyper::Client::new();
        let req = client.post(&url);
        let req = req.header(self.json_content());
        let req = match self.auth {
            Some(ref auth) => {
                let (deliv_user, deliv_token) = auth.auth_headers();
                req.header(deliv_user).header(deliv_token)
            },
            None => req
        };
        if !payload.is_empty() {
            req.body(payload).send()
        } else {
            req.send()
        }
    }

    pub fn put(&self,
               path: &str,
               payload: &str) -> Result<HyperResponse, HttpError> {
        let url = self.api_url(path);
        let mut client = hyper::Client::new();
        let req = client.put(&url);
        let req = req.header(self.json_content());
        let req = match self.auth {
            Some(ref auth) => {
                let (deliv_user, deliv_token) = auth.auth_headers();
                req.header(deliv_user).header(deliv_token)
            },
            None => req
        };
        if !payload.is_empty() {
            req.body(payload).send()
        } else {
            req.send()
        }
    }

    pub fn parse_json(result: Result<HyperResponse, hyper::HttpError>) -> Result<json::Json, DeliveryError> {
        let body = match result {
            Ok(mut b) => {
                let mut body_string = String::new();
                let _x = try!(b.read_to_string(&mut body_string));
                body_string
            },
            Err(e) => return Err(DeliveryError{kind: Kind::HttpError(e),
                                               detail: None})
        };
        Ok(try!(json::Json::from_str(&body)))
    }

    pub fn extract_pretty_json<T: error::Error>(
        result: Result<HyperResponse, T>) -> Result<String, String> {
        let body = match result {
            Ok(mut b) => {
                let mut body_string = String::new();
                match b.read_to_string(&mut body_string) {
                    Ok(_) => body_string,
                    Err(e) => {
                        debug!("extract_pretty_json: {}", e);
                        return Err(String::from_str("response read failed"))
                    }
                }
            },
            Err(e) => {
                debug!("extract_pretty_json: HttpError: {}", e.description());
                return Err(String::from_str("HTTP Error"))
            }
        };
        let json = match json::Json::from_str(&body) {
            Ok(j) => j,
            Err(_) => {
                return Err(String::from_str("invalid JSON"))
            }
        };
        Ok(format!("{}", json.pretty()))
    }

    fn json_content(&self) -> hyper::header::ContentType {
        let mime = mime::Mime(mime::TopLevel::Application,
                              mime::SubLevel::Json, vec![]);
        hyper::header::ContentType(mime)
    }

}

#[derive(Debug)]
pub struct APIAuth {
    user: String,
    token: String
}

impl APIAuth {
    pub fn from_env() -> APIAuth {
        let token = env::var("TOKEN").ok().expect("env missing TOKEN");
        let user = env::var("DEL_USER").ok().expect("env missing DEL_USER");
        APIAuth { user: user, token: token }
    }

    pub fn from_token_store(tstore: TokenStore,
                            server: &str, ent: &str,
                            user: &str) -> Result<APIAuth, DeliveryError> {
        match tstore.lookup(server, ent, user) {
            Some(token) => {
                Ok(APIAuth{ user: String::from_str(user),
                             token: String::from_str(token)})
            },
            None => {
                let msg = format!("server: {}, ent: {}, user: {}",
                                  server, ent, user);
                Err(DeliveryError{ kind: Kind::NoToken,
                                   detail: Some(msg)})
            }
        }
    }

    pub fn user(&self) -> String {
        self.user.clone()
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }
    
    pub fn auth_headers(&self) -> (headers::ChefDeliveryUser,
                                   headers::ChefDeliveryToken) {
        (headers::ChefDeliveryUser(self.user.clone()),
         headers::ChefDeliveryToken(self.token.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::TokenStore;
    use std::env;
    use tempdir::TempDir;
    use utils::path_join_many::PathJoinMany;
    
    #[test]
    fn api_auth() {
        fake_test_env();
        let auth = APIAuth::from_env();
        println!("got auth user: {}, token: {}",
                 auth.user, auth.token);
        assert_eq!("pete", auth.user);
        assert!(auth.token.len() > 4);
    }

    #[test]
    fn http_api_url_test() {
        let mut client = APIClient::new_http("localhost:4343",
                                         "Chef");
        fake_test_env();
        let auth = APIAuth::from_env();        
        client.set_auth(auth);
        let url = client.api_url("foo/bar");
        assert_eq!("http://localhost:4343/api/v0/e/Chef/foo/bar", url)
    }

    #[test]
    fn https_api_url_test() {
        let mut client = APIClient::new_https("localhost:4343",
                                          "Chef");
        fake_test_env();
        let auth = APIAuth::from_env();        
        client.set_auth(auth);
        let url = client.api_url("foo/bar");
        assert_eq!("https://localhost:4343/api/v0/e/Chef/foo/bar", url)
    }

    fn fake_test_env() {
        env::set_var("DEL_USER", "pete");
        env::set_var("TOKEN", "deadbeefcafe");
    }

    #[test]
    fn from_empty_token_store_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let tfile = path.join_many(&["api-tokens"]);
        let tstore = TokenStore::from_file(&tfile).ok().expect("tstore sad");

        // token store is empty so we expect an Err()
        let from_empty = APIAuth::from_token_store(tstore,
                                                   "127.0.0.1", "acme", "bob");
        assert_eq!(true, from_empty.or_else(|e| {
            let msg = "server: 127.0.0.1, ent: acme, user: bob";
            assert_eq!(msg, e.detail().unwrap());
            Err(e)
        }).is_err());
    }

    #[test]
    fn from_non_empty_token_store_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let tfile = path.join_many(&["api-tokens"]);
        let mut tstore = TokenStore::from_file(&tfile).ok().expect("tstore sad");
        let write_result = tstore.write_token("127.0.0.1", "acme", "bob",
                                              "beefbeef");
        assert_eq!(true, write_result.is_ok());
        let auth = APIAuth::from_token_store(tstore, "127.0.0.1", "acme", "bob");
        assert_eq!(true, 
                   auth.and_then(|a| {
                       assert_eq!("bob", a.user());
                       assert_eq!("beefbeef", a.token());
                       Ok(a)
                   }).is_ok());
    }
}
