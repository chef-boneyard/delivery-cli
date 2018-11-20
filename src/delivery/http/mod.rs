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

use config::Config;
use errors::Kind::{ApiError, AuthenticationFailed, EndpointNotFound, ForbiddenRequest,
                   TokenExpired};
use errors::{DeliveryError, Kind};
use http;
use http::token::TokenResponse;
use hyper;
use hyper::client::response::Response as HyperResponse;
use hyper::error::Error as HttpError;
use hyper::mime;
use hyper::status::StatusCode;
use serde_json;
use serde_json::Value as SerdeJson;
use std::env;
use std::fmt;
use std::io::prelude::*;
use std::path::PathBuf;
use token::TokenStore;
use types::DeliveryResult;
use utils::say::sayln;

pub mod change;
mod headers;
pub mod saml;
pub mod token;
pub mod user;

#[derive(Debug)]
enum HProto {
    HTTP,
    HTTPS,
}

impl HProto {
    pub fn from_str(p: &str) -> DeliveryResult<HProto> {
        let lp = p.to_lowercase();
        match lp.as_ref() {
            "http" => Ok(HProto::HTTP),
            "https" => Ok(HProto::HTTPS),
            _ => {
                let msg = format!("unknown protocal: {}", p);
                Err(DeliveryError {
                    kind: Kind::UnsupportedProtocol,
                    detail: Some(msg),
                })
            }
        }
    }
}

impl fmt::Display for HProto {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            &HProto::HTTP => "http",
            &HProto::HTTPS => "https",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
enum HTTPMethod {
    GET,
    PUT,
    POST,
    DELETE,
}

#[derive(Debug)]
pub struct APIClient {
    enterprise: Option<String>,
    api_version: Option<String>,
    proto: HProto,
    host: String,
    auth: Option<APIAuth>,
}

impl APIClient {
    /// Create a new `APIClient` from the specified `Config` instance
    /// for making authenticated requests. Returns an error result if
    /// required configuration values are missing. Expects to find
    /// `server`, `api_port`, `enterprise`, and `user` in the config
    /// along with a token mapped for those values.
    pub fn from_config(config: &Config) -> DeliveryResult<APIClient> {
        APIClient::from_config_no_auth(config).and_then(|mut c| {
            let auth = try!(APIAuth::from_config(&config));
            c.set_auth(auth);
            Ok(c)
        })
    }

    /// Create a new `APIClient` from the specified `Config`
    /// instance. The returned client will not have authentication
    /// data associated with it. The call will read `server`,
    /// `api_port`, and `enterprise` from the specified config and
    /// raise an error if any of these values are unset.
    pub fn from_config_no_auth(config: &Config) -> DeliveryResult<APIClient> {
        APIClient::from_config_with_basic_routing(config).and_then(|mut c| {
            let ent = try!(config.enterprise());
            c.set_enterprise(&ent);
            c.set_api_version("v0");
            Ok(c)
        })
    }

    // Create a new `APIClient` from the specified `Config`
    // instance that makes no assumptions about prepending
    // your routes with the enterprise or API version.
    // Use to make unauthenticated requests where you specify
    // the full route.
    pub fn from_config_with_basic_routing(config: &Config) -> DeliveryResult<APIClient> {
        let host = try!(config.api_base_resource());
        let proto_str = try!(config.api_protocol());
        let proto = try!(HProto::from_str(&proto_str));
        Ok(APIClient::new(proto, &host))
    }

    /// Create a new `APIClient` using HTTP attached to the enterprise
    /// given by `ent`.
    pub fn new_http(host: &str, ent: &str) -> APIClient {
        let mut api_client = APIClient::new(HProto::HTTP, host);
        api_client.set_enterprise(&ent);
        api_client.set_api_version("v0");
        api_client
    }

    /// Create a new `APIClient` using HTTPS attached to the
    /// enterprise given by `ent`.
    pub fn new_https(host: &str, ent: &str) -> APIClient {
        let mut api_client = APIClient::new(HProto::HTTPS, host);
        api_client.set_enterprise(&ent);
        api_client.set_api_version("v0");
        api_client
    }

    /// Parse the HyperResponse coming from a APIClient Request that
    /// the server sends back when hitting an endpoints. It will
    /// interprete the response and return Ok() with a tuple inside:
    ///
    /// (StatusCode, Option<String>)
    ///
    /// Where:
    ///     -> StatusCode:      The request response code
    ///     -> Option<String>:  Optional content of the request
    ///
    /// Or throwing an Err() explaining the reason the request failed
    ///
    // NOTE: We consider a '409 Conflict' to be Ok() because this code
    // is returned when we are trying to create something that already
    // exist, so we do not want to explode in that case.
    //
    // If the caller needs to interpret that code as a failure it can
    // easily do it as the following example:
    //
    // # Example:
    //
    // ```rust
    // match try!(APIClient::parse_response(response)) {
    //     (Conflict, _content) => {
    //         sayln("error", "There was a conflict!");
    //     },
    //     (code, _content) => {
    //         sayln("success", "All good.");
    //     },
    // }
    // ```
    pub fn parse_response(
        mut response: HyperResponse,
    ) -> DeliveryResult<(StatusCode, Option<String>)> {
        use self::StatusCode::Ok as OkCode;
        use self::StatusCode::{Conflict, Created, Forbidden, NoContent, NotFound, Unauthorized};
        debug!("Parsing response with status: {:?}", response.status);
        match response.status {
            NoContent | Created | Conflict => Ok((response.status, None)),
            OkCode => {
                let pretty_json = try!(APIClient::extract_pretty_json(&mut response));
                Ok((response.status, Some(pretty_json)))
            }
            NotFound => {
                let msg = format!("Unable to access endpoint: {}", response.url);
                Err(DeliveryError::throw(EndpointNotFound, Some(msg)))
            }
            Forbidden => {
                let msg = "The user is not authorized to perform this action.\nContact \
                           an administrator to grant you with appropriate permissions to \
                           proceed."
                    .to_string();
                Err(DeliveryError::throw(ForbiddenRequest, Some(msg)))
            }
            Unauthorized => {
                let detail = try!(APIClient::extract_pretty_json(&mut response));
                if TokenResponse::parse_token_expired(&detail) {
                    return Err(DeliveryError::throw(TokenExpired, None));
                }
                let msg = format!(
                    "Request lacks valid authentication credentials.\n\
                     Detail:\n{}",
                    detail
                );
                Err(DeliveryError::throw(AuthenticationFailed, Some(msg)))
            }
            error_code @ _ => {
                let msg = format!("Request returned: '{}'", error_code);
                let mut detail = String::new();
                let e = response.read_to_string(&mut detail).and(Ok(detail));
                Err(DeliveryError::throw(ApiError(error_code, e), Some(msg)))
            }
        }
    }

    fn new(proto: HProto, host: &str) -> APIClient {
        APIClient {
            api_version: None,
            proto: proto,
            host: String::from(host),
            enterprise: None,
            auth: None,
        }
    }

    pub fn set_auth(&mut self, auth: APIAuth) {
        self.auth = Some(auth);
    }

    pub fn set_enterprise(&mut self, ent: &str) {
        self.enterprise = Some(String::from(ent))
    }

    pub fn set_api_version(&mut self, api_version: &str) {
        self.api_version = Some(String::from(api_version))
    }

    pub fn get_auth_from_home(
        &mut self,
        server: &str,
        ent: &str,
        user: &str,
    ) -> DeliveryResult<APIAuth> {
        match TokenStore::from_home() {
            Ok(tstore) => APIAuth::from_token_store(tstore, server, ent, user),
            Err(e) => Err(e),
        }
    }

    pub fn api_url(&self, path: &str) -> String {
        let mut request_path = format!("{}://{}", self.proto, self.host);

        if let Some(ref version) = self.api_version {
            request_path += &format!("/api/{}", version);
        }

        if let Some(ref ent) = self.enterprise {
            request_path += &format!("/e/{}", ent);
        }

        request_path += &format!("/{}", path);

        request_path
    }

    pub fn get(&self, path: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::GET, path, "")
    }

    pub fn delete(&self, path: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::DELETE, path, "")
    }

    pub fn put(&self, path: &str, payload: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::PUT, path, payload)
    }

    pub fn post(&self, path: &str, payload: &str) -> Result<HyperResponse, HttpError> {
        self.req_with_body(HTTPMethod::POST, path, payload)
    }

    /// Send a request using the specified HTTP verb. If `payload` is
    /// an empty string, no request body will be sent. This could be an
    /// `Options<String>` but (I think) keeping the simple `&str`
    /// avoids an allocation.
    fn req_with_body(
        &self,
        http_method: HTTPMethod,
        path: &str,
        payload: &str,
    ) -> Result<HyperResponse, HttpError> {
        let url = self.api_url(path);
        let client = hyper::Client::new();
        let req = match http_method {
            HTTPMethod::GET => client.get(&url),
            HTTPMethod::PUT => client.put(&url),
            HTTPMethod::POST => client.post(&url),
            HTTPMethod::DELETE => client.delete(&url),
        };
        let req = req.header(self.json_content());
        let req = match self.auth {
            Some(ref auth) => {
                let (deliv_user, deliv_token) = auth.auth_headers();
                req.header(deliv_user).header(deliv_token)
            }
            None => req,
        };
        debug!(
            "Request: {:?} Path: {:?} Payload: {:?}",
            http_method, path, payload
        );
        if payload.is_empty() {
            req.send()
        } else {
            req.body(payload).send()
        }
    }

    pub fn pipeline_exists(&self, org: &str, proj: &str, pipe: &str) -> bool {
        let path = format!("orgs/{}/projects/{}/pipelines/{}", org, proj, pipe);
        match self.get(&path) {
            Ok(res) => match res.status {
                StatusCode::Ok => {
                    return true;
                }
                _ => {
                    return false;
                }
            },
            Err(e) => {
                sayln("red", &format!("pipeline_exists: HttpError: {:?}", e));
                return false;
            }
        }
    }

    pub fn project_exists(&self, org: &str, proj: &str) -> bool {
        let path = format!("orgs/{}/projects/{}", org, proj);
        match self.get(&path) {
            Ok(res) => match res.status {
                StatusCode::Ok => {
                    return true;
                }
                _ => {
                    return false;
                }
            },
            Err(e) => {
                sayln("red", &format!("project_exists: HttpError: {:?}", e));
                return false;
            }
        }
    }

    pub fn create_delivery_project(&self, org: &str, proj: &str) -> DeliveryResult<StatusCode> {
        let path = format!("orgs/{}/projects", org);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let payload = format!("{{\"name\":\"{}\"}}", proj);
        Self::parse_response(self.post(&path, &payload)?).map(|(code, _)| code)
    }

    pub fn create_github_project(
        &self,
        org: &str,
        proj: &str,
        repo_name: &str,
        git_org: &str,
        pipe: &str,
        ssl: bool,
    ) -> DeliveryResult<StatusCode> {
        let path = format!("orgs/{}/github-projects", org);
        let payload = format!(
            "{{\
             \"name\":\"{}\",\
             \"scm\":{{\
             \"type\":\"github\",\
             \"project\":\"{}\",\
             \"organization\":\"{}\",\
             \"branch\":\"{}\",\
             \"verify_ssl\": {}\
             }}\
             }}",
            proj, repo_name, git_org, pipe, ssl
        );
        Self::parse_response(self.post(&path, &payload)?).map(|(code, _)| code)
    }

    fn get_scm_server_config(&self, scm: &str) -> DeliveryResult<Vec<SerdeJson>> {
        let json = try!(APIClient::parse_json(self.get("scm-providers")));
        debug!("Endpoint[scm-providers]: {:?}", json);
        if let Some(data) = json.as_array() {
            for obj in data.iter() {
                if let Some(scp) = obj.as_object() {
                    let name = scp.get("name")
                        .expect("Missing 'name' field for /scm-providers endpoint")
                        .as_str()
                        .unwrap();
                    if name == scm {
                        debug!("Found SCM {} config: {:?}", scm, scp);
                        let scp_config = scp.get("scmSetupConfigs")
                            .expect("Missing 'scmSetupConfigs' field for /scm-providers endpoint")
                            .as_array()
                            .unwrap();
                        debug!("scmSetupConfigs: {:?}", scp_config);
                        return Ok(scp_config.clone());
                    }
                }
            }
        }
        Err(DeliveryError {
            kind: Kind::ExpectedJsonString,
            detail: Some(format!(
                "Unable to find {:?} SCM Config in Delivery Server.\n\
                 JSON Output: {:?}",
                scm, json
            )),
        })
    }

    pub fn get_github_server_config(&self) -> DeliveryResult<Vec<SerdeJson>> {
        self.get_scm_server_config("GitHub")
    }

    pub fn get_bitbucket_server_config(&self) -> DeliveryResult<Vec<SerdeJson>> {
        self.get_scm_server_config("Bitbucket")
    }

    pub fn create_bitbucket_project(
        &self,
        org: &str,
        proj: &str,
        repo_name: &str,
        project_key: &str,
        pipe: &str,
    ) -> DeliveryResult<StatusCode> {
        let path = format!("orgs/{}/bitbucket-projects", org);
        let payload = format!(
            "{{\
             \"name\":\"{}\",\
             \"scm\":{{\
             \"type\":\"bitbucket\",\
             \"repo_name\":\"{}\",\
             \"project_key\":\"{}\",\
             \"pipeline_branch\":\"{}\"\
             }}\
             }}",
            proj, repo_name, project_key, pipe
        );
        Self::parse_response(self.post(&path, &payload)?).map(|(code, _)| code)
    }

    pub fn create_pipeline(
        &self,
        org: &str,
        proj: &str,
        pipe: &str,
        base: Option<&str>,
    ) -> DeliveryResult<StatusCode> {
        let path = format!("orgs/{}/projects/{}/pipelines", org, proj);

        // We unwrap the provided base branch, if None we default to `master`
        let base_branch = base.unwrap_or("master");
        let payload = format!("{{\"name\":\"{}\",\"base\":\"{}\"}}", pipe, base_branch);

        Self::parse_response(self.post(&path, &payload)?).map(|(code, _)| code)
    }

    pub fn parse_json(result: Result<HyperResponse, HttpError>) -> DeliveryResult<SerdeJson> {
        let body = match result {
            Ok(mut b) => {
                let mut body_string = String::new();
                let _x = try!(b.read_to_string(&mut body_string));
                body_string
            }
            Err(e) => {
                return Err(DeliveryError {
                    kind: Kind::HttpError(e),
                    detail: None,
                })
            }
        };
        Ok(serde_json::from_str(&body)?)
    }

    pub fn extract_pretty_json(resp: &mut HyperResponse) -> DeliveryResult<String> {
        let mut body = String::new();
        resp.read_to_string(&mut body)?;
        debug!("Status: {:?} Body: {:?}", resp.status, body);
        let json: SerdeJson = serde_json::from_str(&body)?;
        Ok(serde_json::to_string_pretty(&json)?)
    }

    fn json_content(&self) -> hyper::header::ContentType {
        let mime = mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, vec![]);
        hyper::header::ContentType(mime)
    }
}

#[derive(Debug)]
pub struct APIAuth {
    user: String,
    token: String,
}

impl APIAuth {
    pub fn from_env() -> APIAuth {
        let token = env::var("TOKEN").ok().expect("env missing TOKEN");
        let user = env::var("DEL_USER").ok().expect("env missing DEL_USER");
        APIAuth {
            user: user,
            token: token,
        }
    }

    /// Create an `APIAuth` struct from the specified `Config`
    /// instance. Expects to find valid values for `server`,
    /// `enterprise`, and `user`.
    /// Reads API tokens from `$HOME/.delivery/api-tokens`.
    /// Lookup for the stored token, if it does not exist request it.
    pub fn from_config(config: &Config) -> DeliveryResult<APIAuth> {
        if !try!(http::token::verify(&config)) {
            sayln("red", "Token expired");
            return APIAuth::from_token_request(config);
        }
        let tstore = match config.token_file {
            Some(ref f) => {
                let file = PathBuf::from(f);
                try!(TokenStore::from_file(&file))
            }
            None => try!(TokenStore::from_home()),
        };
        let api_server = try!(config.api_base_resource());
        let ent = try!(config.enterprise());
        let user = try!(config.user());
        APIAuth::from_token_store(tstore, &api_server, &ent, &user).or_else(|e| {
            debug!("Ignoring {:?}\nRequesting token from config", e);
            APIAuth::from_token_request(&config)
        })
    }

    pub fn from_token_store(
        tstore: TokenStore,
        server: &str,
        ent: &str,
        user: &str,
    ) -> DeliveryResult<APIAuth> {
        match tstore.lookup(server, ent, user) {
            Some(token) => {
                debug!("Token found");
                Ok(APIAuth {
                    user: String::from(user),
                    token: token.clone(),
                })
            }
            None => {
                debug!("Token not found");
                let msg = format!("server: {}, ent: {}, user: {}", server, ent, user);
                Err(DeliveryError {
                    kind: Kind::NoToken,
                    detail: Some(msg),
                })
            }
        }
    }

    pub fn from_token_request(config: &Config) -> DeliveryResult<APIAuth> {
        let user = try!(config.user());
        let interactive = !config.non_interactive.unwrap_or(false);
        if interactive {
            let token = try!(TokenStore::request_token(&config));
            debug!("APIAuth from_token_request: {:?}@{:?}", user, token);
            Ok(APIAuth {
                user: user.clone(),
                token: token.clone(),
            })
        } else {
            let msg = format!(
                "Unable to request token due to --no-interactive \
                 flag.\nTry `delivery token` to create one"
            );
            Err(DeliveryError {
                kind: Kind::NoToken,
                detail: Some(msg),
            })
        }
    }

    pub fn user(&self) -> String {
        self.user.clone()
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }

    pub fn auth_headers(&self) -> (headers::ChefDeliveryUser, headers::ChefDeliveryToken) {
        (
            headers::ChefDeliveryUser(self.user.clone()),
            headers::ChefDeliveryToken(self.token.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use config::Config;
    use std::env;
    use tempdir::TempDir;
    use token::TokenStore;
    use utils::path_join_many::PathJoinMany;

    #[test]
    fn api_auth() {
        fake_test_env();
        let auth = APIAuth::from_env();
        println!("got auth user: {}, token: {}", auth.user, auth.token);
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
    fn from_config_no_auth_override_api_version_test() {
        let config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth");

        let mut client = APIClient::from_config_no_auth(&config).unwrap();
        client.set_api_version("v1");
        let url = client.api_url("foo");
        assert_eq!("https://earth/api/v1/e/ncc-1701/foo", url)
    }

    #[test]
    fn from_config_with_basic_routing_test() {
        let config = Config::default().set_server("earth");
        let client = APIClient::from_config_with_basic_routing(&config).unwrap();
        let url = client.api_url("api/_status");
        assert_eq!("https://earth/api/_status", url)
    }

    #[test]
    fn from_config_needs_user() {
        let mut config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth");
        config.non_interactive = Some(true);

        match APIClient::from_config(&config) {
            Ok(_) => assert!(false),
            Err(e) => {
                let m = e.detail().unwrap();
                assert_eq!(
                    "User not set; try --user or set it in your \
                     .toml config file",
                    &m
                );
            }
        };
    }

    #[test]
    fn from_config_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let token_file = path.join_many(&["api-tokens"]);
        let mut config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth")
            .set_user("kirk")
            .set_token_file(token_file.to_str().unwrap());
        config.non_interactive = Some(true);

        let mut tstore = TokenStore::from_file(&token_file).ok().expect("tstore sad");
        let write_result = tstore.write_token("earth", "ncc-1701", "kirk", "cafecafe");
        assert_eq!(true, write_result.is_ok());

        // Turning this test into a verification instead since `from_config`
        // now validates that the token extracted from the tstore is valid.
        // That means that it hits an endpoint and we can't mock it in this test.
        let client = APIClient::from_config(&config);
        assert!(client.is_err());

        // Instead we use `from_config_no_auth` that doesn't validate the token
        let client_from_store = APIClient::from_config_no_auth(&config).unwrap();
        let url = client_from_store.api_url("foo");
        assert_eq!("https://earth/api/v0/e/ncc-1701/foo", url)
    }

    #[test]
    fn http_api_url_test() {
        let mut client = APIClient::new_http("localhost:4343", "Chef");
        fake_test_env();
        let auth = APIAuth::from_env();
        client.set_auth(auth);
        let url = client.api_url("foo/bar");
        assert_eq!("http://localhost:4343/api/v0/e/Chef/foo/bar", url)
    }

    #[test]
    fn https_api_url_test() {
        let mut client = APIClient::new_https("localhost:4343", "Chef");
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
        let from_empty = APIAuth::from_token_store(tstore, "127.0.0.1", "acme", "bob");

        assert_eq!(
            true,
            from_empty
                .or_else(|e| {
                    let msg = "server: 127.0.0.1, ent: acme, user: bob";
                    assert_eq!(msg, e.detail().unwrap());
                    Err(e)
                })
                .is_err()
        );
    }

    #[test]
    fn from_non_empty_token_store_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let tfile = path.join_many(&["api-tokens"]);
        let mut tstore = TokenStore::from_file(&tfile).ok().expect("tstore sad");
        let write_result = tstore.write_token("127.0.0.1", "acme", "bob", "beefbeef");
        assert_eq!(true, write_result.is_ok());
        let auth = APIAuth::from_token_store(tstore, "127.0.0.1", "acme", "bob");
        assert_eq!(
            true,
            auth.and_then(|a| {
                assert_eq!("bob", a.user());
                assert_eq!("beefbeef", a.token());
                Ok(a)
            }).is_ok()
        );
    }

    #[test]
    fn api_auth_from_config_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let token_file = path.join_many(&["api-tokens"]);
        let mut config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth")
            .set_user("kirk")
            .set_token_file(token_file.to_str().unwrap());
        config.non_interactive = Some(true);

        let mut tstore = TokenStore::from_file(&token_file).ok().expect("tstore sad");
        let write_result = tstore.write_token("earth", "ncc-1701", "kirk", "cafecafe");
        assert_eq!(true, write_result.is_ok());

        let auth = APIAuth::from_config(&config);

        // Turning this test into a verification instead since `from_config`
        // now validatates that the token extracted from the tstore is valid.
        // That means that it hits an endpoint and we can't mock it in this test.
        assert!(auth.is_err());

        // Instead we use `from_token_store` that doesn't validate the token
        let auth_from_tstore = APIAuth::from_token_store(tstore, "earth", "ncc-1701", "kirk");
        assert_eq!(
            true,
            auth_from_tstore
                .and_then(|a| {
                    assert_eq!("kirk", a.user());
                    assert_eq!("cafecafe", a.token());
                    Ok(a)
                })
                .is_ok()
        );
    }

    #[test]
    fn api_auth_from_config_when_missing_test() {
        let mut config = Config::default()
            .set_enterprise("ncc-1701")
            .set_server("earth");
        config.non_interactive = Some(true);

        // NOTE: for now, the use of the HOME environment variable
        // makes this test unsafe for parallel execution.
        // let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        // let path = tempdir.path();
        // env::set_var("HOME", path);

        let auth = APIAuth::from_config(&config);
        assert_eq!(
            true,
            auth.or_else(|e| {
                println!("e: {:?}", e);
                let detail = &e.detail.unwrap();
                let expect = "User not set; try --user or set it in \
                              your .toml config file";
                assert_eq!(expect, detail);
                Err(1)
            }).is_err()
        );
    }

    mod http_request {
        use super::*;
        use mockito::SERVER_ADDRESS;

        fn client() -> APIClient {
            APIClient::new_http(SERVER_ADDRESS, "gamer")
        }

        macro_rules! mock_endpoints {
            () => {
                let _m1 = mock("GET", "/api/v0/e/gamer/orgs")
                    .with_status(200)
                    .match_header("Content-Type", "application/json")
                    .with_body("{}")
                    .create();
                let _m2 = mock("GET", "/api/v0/e/gamer/not_found")
                    .with_status(404)
                    .create();
                let _m3 = mock("POST", "/api/v0/e/gamer/orgs")
                    .with_status(201)
                    .create();
                let _m4 = mock("POST", "/api/v0/e/gamer/orgs/zelda/projects")
                    .with_status(409)
                    .create();
                let _m5 = mock("DELETE", "/api/v0/e/gamer/orgs/ganondorf")
                    .with_status(204)
                    .create();
                let _m6 = mock("GET", "/api/v0/e/gamer/users")
                    .with_status(401)
                    .match_header("Content-Type", "application/json")
                    .with_body("{\"error\": \"unauthorized\"}")
                    .create();
                let _m7 = mock("POST", "/api/v0/e/gamer/internal-users")
                    .with_status(401)
                    .match_header("Content-Type", "application/json")
                    .with_body("{\"error\": \"token_expired\"}")
                    .create();
            };
        }

        mod parse_response {
            use super::{client, APIClient};
            use mockito::mock;

            #[test]
            fn ok() {
                mock_endpoints!();
                let response = client().get("orgs").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_ok());
                let (code, content) = tuple.unwrap();
                assert!(content.is_some());
                assert_eq!(content.unwrap(), "{}");
                assert_eq!(code, super::StatusCode::Ok);
            }

            #[test]
            fn no_content() {
                mock_endpoints!();
                let response = client().delete("orgs/ganondorf").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_ok());
                let (code, content) = tuple.unwrap();
                assert!(content.is_none());
                assert_eq!(code, super::StatusCode::NoContent);
            }

            #[test]
            fn created() {
                mock_endpoints!();
                let response = client().post("orgs", "name: zelda").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_ok());
                let (code, content) = tuple.unwrap();
                assert!(content.is_none());
                assert_eq!(code, super::StatusCode::Created);
            }

            #[test]
            fn conflict_that_we_consider_as_ok() {
                mock_endpoints!();
                let response = client()
                    .post("orgs/zelda/projects", "name: already_exist")
                    .unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_ok());
                let (code, content) = tuple.unwrap();
                assert!(content.is_none());
                assert_eq!(code, super::StatusCode::Conflict);
            }

            #[test]
            fn not_found() {
                mock_endpoints!();
                let response = client().get("not_found").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_err());
                let error = tuple.unwrap_err();
                assert_eq!(
                    error.detail,
                    Some(format!(
                        "Unable to access endpoint: {}",
                        client().api_url("not_found")
                    ))
                );
                assert_enum!(error.kind, super::EndpointNotFound);
            }

            #[test]
            fn unauthorized_token_expired() {
                mock_endpoints!();
                let response = client().post("internal-users", "name: link").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_err());
                let error = tuple.unwrap_err();
                assert!(error.detail.is_none());
                assert_enum!(error.kind, super::TokenExpired);
            }

            #[test]
            fn unauthorized() {
                mock_endpoints!();
                let response = client().get("users").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_err());
                let error = tuple.unwrap_err();
                let msg = "Request lacks valid authentication credentials.\n\
                           Detail:\n{\n  \"error\": \"unauthorized\"\n}"
                    .to_string();
                assert_eq!(error.detail, Some(msg));
                assert_enum!(error.kind, super::AuthenticationFailed);
            }

            #[test]
            fn any_other_request() {
                mock_endpoints!();
                let response = client().get("odd-endpoint").unwrap();
                let tuple = APIClient::parse_response(response);
                assert!(tuple.is_err());
                let error = tuple.unwrap_err();
                assert_eq!(
                    error.detail,
                    Some("Request returned: \'501 Not Implemented\'".to_string())
                );
                match error.kind {
                    super::ApiError(code, _) => {
                        assert_eq!(code, super::StatusCode::NotImplemented);
                    }
                    _ => panic!("Wrong kind of error!"),
                };
            }
        }
    }
}
