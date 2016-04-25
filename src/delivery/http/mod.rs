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
use std::path::PathBuf;
use hyper;
use hyper::status::StatusCode;
use hyper::client::response::Response as HyperResponse;
use hyper::error::Error as HttpError;
use mime;
use rustc_serialize::json;
use std::io::prelude::*;
use errors::{DeliveryError, Kind};
use token::TokenStore;
use utils::say::sayln;
use config::Config;

mod headers;
pub mod token;
pub mod change;

#[derive(Debug)]
enum HProto {
    HTTP,
    HTTPS
}

impl HProto {
    pub fn from_str(p: &str) -> Result<HProto, DeliveryError> {
        let lp = p.to_lowercase();
        match lp.as_ref() {
            "http" => Ok(HProto::HTTP),
            "https" => Ok(HProto::HTTPS),
            _ => {
                let msg = format!("unknown protocal: {}", p);
                Err(DeliveryError{ kind: Kind::UnsupportedProtocol,
                               detail: Some(msg) })
            }
        }
    }
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

#[derive(Debug)]
enum HTTPMethod {
    GET,
    PUT,
    POST,
    DELETE
}

#[derive(Debug)]
pub struct APIClient {
    proto: HProto,
    host: String,
    enterprise: String,
    auth: Option<APIAuth>
}

impl APIClient {
    /// Create a new `APIClient` from the specified `Config` instance
    /// for making authenticated requests. Returns an error result if
    /// required configuration values are missing. Expects to find
    /// `server`, `api_port`, `enterprise`, and `user` in the config
    /// along with a token mapped for those values.
    pub fn from_config(config: &Config) -> Result<APIClient, DeliveryError> {
        APIClient::from_config_no_auth(config).and_then(|mut c| {
            let auth = try!(APIAuth::from_config(&config));
            c.set_auth(auth);
            Ok(c)
        })
        // Ok(client)
    }

    /// Create a new `APIClient` from the specified `Config`
    /// instance. The returned client will not have authentication
    /// data associated with it. The call will read `server`,
    /// `api_port`, and `enterprise` from the specified config and
    /// raise an error if any of these values are unset.
    pub fn from_config_no_auth(config: &Config)
                               -> Result<APIClient, DeliveryError> {
        let host = try!(config.api_host_and_port());
        let ent = try!(config.enterprise());
        let proto_str = try!(config.api_protocol());
        let proto = try!(HProto::from_str(&proto_str));
        Ok(APIClient::new(proto, &host, &ent))
    }

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
            host: String::from(host),
            enterprise: String::from(ent),
            auth: None
        }
    }

    /// Parse the HyperResponse coming from the APIClient Request
    /// that Delivery sends back when hitting the endpoints
    pub fn parse_response(&self, mut response: HyperResponse) -> Result<(), DeliveryError> {
        match response.status {
            StatusCode::NoContent => Ok(()),
            StatusCode::Created => {
                sayln("green", " created");
                Ok(())
            },
            StatusCode::Conflict => {
                sayln("white", " already exists.");
                Ok(())
            },
            StatusCode::Unauthorized => {
                // Verify if the Token has expired
                let detail = try!(APIClient::extract_pretty_json(&mut response));
                match detail.find("token_expired") {
                    Some(_) => Err(DeliveryError{ kind: Kind::TokenExpired, detail: None}),
                    None => {
                        let msg = "API request returned 401".to_string();
                        Err(DeliveryError{ kind: Kind::AuthenticationFailed,
                                           detail: Some(msg)})
                    }
                }
            },
            error_code @ _ => {
                let msg = format!("API request returned {}",
                                  error_code);
                let mut detail = String::new();
                let e = match response.read_to_string(&mut detail) {
                    Ok(_) => Ok(detail),
                    Err(e) => Err(e)
                };
                Err(DeliveryError{ kind: Kind::ApiError(error_code, e),
                                   detail: Some(msg)})
            }
        }
    }

    pub fn get_auth_from_home(&mut self, server: &str, ent: &str,
                              user: &str) -> Result<APIAuth, DeliveryError> {
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

    pub fn get(&self, path: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::GET, path, "")
    }

    pub fn delete(&self, path: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::DELETE, path, "")
    }

    pub fn put(&self, path: &str,
               payload: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::PUT, path, payload)
    }

    pub fn post(&self, path: &str,
                payload: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::POST, path, payload)
    }

    // Send a request using the specified HTTP verb. If `payload` is
    // an empty string, no request body will be sent. This could be an
    // `Options<String>` but (I think) keeping the simple `&str`
    // avoids an allocation.
    fn req_with_body(&self,
                     http_method: HTTPMethod,
                     path: &str,
                     payload: &str) -> Result<HyperResponse, HttpError> {
        let url = self.api_url(path);
        let client = hyper::Client::new();
        let req = match http_method {
            HTTPMethod::GET    => client.get(&url),
            HTTPMethod::PUT    => client.put(&url),
            HTTPMethod::POST   => client.post(&url),
            HTTPMethod::DELETE => client.delete(&url)
        };
        let req = req.header(self.json_content());
        let req = match self.auth {
            Some(ref auth) => {
                let (deliv_user, deliv_token) = auth.auth_headers();
                req.header(deliv_user).header(deliv_token)
            },
            None => req
        };
        debug!("Request: {:?} Path: {:?} Payload: {:?}",
                http_method, path, payload);
        if !payload.is_empty() {
            req.body(payload).send()
        } else {
            req.send()
        }
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

    pub fn create_delivery_project(&self, org: &str,
                                   proj: &str) -> Result<(), DeliveryError> {
        let path = format!("orgs/{}/projects", org);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let payload = format!("{{\"name\":\"{}\"}}", proj);
        self.parse_response(try!(self.post(&path, &payload)))
    }

    pub fn create_github_project(&self, org: &str, proj: &str,
                                repo_name: &str, git_org: &str, pipe: &str,
                                ssl: bool) -> Result<(), DeliveryError> {
        let path = format!("orgs/{}/github-projects", org);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let payload = format!("{{\
                                \"name\":\"{}\",\
                                \"scm\":{{\
                                    \"type\":\"github\",\
                                    \"project\":\"{}\",\
                                    \"organization\":\"{}\",\
                                    \"branch\":\"{}\",\
                                    \"verify_ssl\": {}\
                                }}\
                              }}", proj, repo_name, git_org, pipe, ssl);
        self.parse_response(try!(self.post(&path, &payload)))
    }

    pub fn create_bitbucket_project(&self, org: &str, proj: &str,
                                    repo_name: &str, project_key: &str,
                                    pipe: &str) -> Result<(), DeliveryError> {
        let path = format!("orgs/{}/bitbucket-projects", org);
        let payload = format!("{{\
                                \"name\":\"{}\",\
                                \"scm\":{{\
                                    \"type\":\"bitbucket\",\
                                    \"repo_name\":\"{}\",\
                                    \"project_key\":\"{}\",\
                                    \"pipeline_branch\":\"{}\"\
                                }}\
                              }}", proj, repo_name, project_key, pipe);
        self.parse_response(try!(self.post(&path, &payload)))
    }

    pub fn create_pipeline(&self,
                           org: &str,
                           proj: &str,
                           pipe: &str) -> Result<(), DeliveryError> {
        let path = format!("orgs/{}/projects/{}/pipelines", org, proj);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let base_branch = "master";
        let payload = format!("{{\"name\":\"{}\",\"base\":\"{}\"}}",
                              pipe, base_branch);
        self.parse_response(try!(self.post(&path, &payload)))
    }

    pub fn parse_json(result: Result<HyperResponse, HttpError>) -> Result<json::Json, DeliveryError> {
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

    pub fn extract_pretty_json(resp: &mut HyperResponse) ->
        Result<String, DeliveryError> {
            let mut body = String::new();
            try!(resp.read_to_string(&mut body));
            let json = try!(json::Json::from_str(&body));
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

    /// Create an `APIAuth` struct from the specified `Config`
    /// instance. Expects to find valid values for `server`,
    /// `enterprise`, and `user`.
    /// Reads API tokens from `$HOME/.delivery/api-tokens`.
    /// Lookup for the stored token, if it does not exist request it.
    pub fn from_config(config: &Config) -> Result<APIAuth, DeliveryError> {
        let tstore = match config.token_file {
            Some(ref f) => {
                let file = PathBuf::from(f);
                try!(TokenStore::from_file(&file))
            },
            None => try!(TokenStore::from_home())
        };
        let api_server = try!(config.api_host_and_port());
        let ent = try!(config.enterprise());
        let user = try!(config.user());
        let api_auth = APIAuth::from_token_store(tstore, &api_server,
                                                 &ent, &user);
        api_auth.or_else(|e| {
            let interactive = !config.non_interactive.unwrap_or(false);
            if interactive {
                let token = try!(TokenStore::request_token(&config));
                Ok(APIAuth{ user: user.clone(),
                            token: token.clone()})
            } else {
                Err(e)
            }
        })
    }

    pub fn from_token_store(tstore: TokenStore,
                            server: &str, ent: &str,
                            user: &str) -> Result<APIAuth, DeliveryError> {
        match tstore.lookup(server, ent, user) {
            Some(token) => {
                Ok(APIAuth{ user: String::from(user),
                            token: token.clone()})
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
    use config::Config;

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
    fn from_config_no_auth_test() {
        let config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth");

        let client = APIClient::from_config_no_auth(&config).unwrap();
        let url = client.api_url("foo");
        assert_eq!("https://earth/api/v0/e/ncc-1701/foo", url)
    }

    #[test]
    fn from_config_needs_user() {
        let config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth");

        match APIClient::from_config(&config) {
            Ok(_) => assert!(false),
            Err(e) => {
                let m = e.detail().unwrap();
                assert_eq!("User not set; try --user or set it in your \
                           .toml config file", &m);
            }
        };

    }

    #[test]
    fn from_config_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let token_file = path.join_many(&["api-tokens"]);
        let config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth")
            .set_user("kirk")
            .set_token_file(token_file.to_str().unwrap());

        let mut tstore = TokenStore::from_file(&token_file).ok().expect("tstore sad");
        let write_result = tstore.write_token("earth", "ncc-1701", "kirk",
                                              "cafecafe");
        assert_eq!(true, write_result.is_ok());

        let client = APIClient::from_config(&config).unwrap();
        let url = client.api_url("foo");
        assert_eq!("https://earth/api/v0/e/ncc-1701/foo", url)
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

    #[test]
    fn api_auth_from_config_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let token_file = path.join_many(&["api-tokens"]);
        let config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth")
            .set_user("kirk")
            .set_token_file(token_file.to_str().unwrap());

        let mut tstore = TokenStore::from_file(&token_file).ok().expect("tstore sad");
        let write_result = tstore.write_token("earth", "ncc-1701", "kirk",
                                              "cafecafe");
        assert_eq!(true, write_result.is_ok());

        let auth = APIAuth::from_config(&config);

        assert_eq!(true,
                   auth.and_then(|a| {
                       assert_eq!("kirk", a.user());
                       assert_eq!("cafecafe", a.token());
                       Ok(a)
                   }).is_ok());
    }

    #[test]
    fn api_auth_from_config_when_missing_test() {
        let config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth");

        // NOTE: for now, the use of the HOME environment variable
        // makes this test unsafe for parallel execution.
        // let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        // let path = tempdir.path();
        // env::set_var("HOME", path);

        let auth = APIAuth::from_config(&config);
        assert_eq!(true,
                   auth.or_else(|e| {
                       println!("e: {:?}", e);
                       let detail = &e.detail.unwrap();
                       let expect = "User not set; try --user or set it in \
                                     your .toml config file";
                       assert_eq!(expect, detail);
                       Err(1)
                   }).is_err());
    }
}
