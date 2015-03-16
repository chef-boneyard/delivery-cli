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
mod headers;

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
    auth: APIAuth
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
            auth: APIAuth::from_env()
        }
    }

    pub fn api_url(&self, path: &str) -> String {
        format!("{}://{}/api/v0/e/{}/{}",
                self.proto, self.host, self.enterprise, path)
    }

    pub fn create_project(&self,
                          org: &str,
                          proj: &str) -> Result<HyperResponse, DelivError> {
        let path = format!("orgs/{}/projects", org);
        // FIXME: we'd like to use the native struct->json stuff, but
        // seeing link issues.
        let payload = format!("{{\"name\":\"{}\"}}", proj);
        match self.post(path.as_slice(), payload.as_slice()) {
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
                match self.post(path.as_slice(), payload.as_slice()) {
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
        let req = client.get(url.as_slice());
        let auth = APIAuth::from_env();
        let (deliv_user, deliv_token) = auth.auth_headers();
        let req = req.header(self.json_content())
            .header(deliv_user)
            .header(deliv_token);
        req.send()
    }

    pub fn post(&self,
                path: &str,
                payload: &str) -> Result<HyperResponse, HttpError> {
        let url = self.api_url(path);
        let mut client = hyper::Client::new();
        let req = client.post(url.as_slice());
        let auth = APIAuth::from_env();
        let (deliv_user, deliv_token) = auth.auth_headers();
        let req = req.header(self.json_content())
            .header(deliv_user)
            .header(deliv_token);
        if !payload.is_empty() {
            req.body(payload).send()
        } else {
            req.send()
        }
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
        let json = match json::Json::from_str(body.as_slice()) {
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
struct APIAuth {
    user: String,
    token: String
}

impl APIAuth {
    pub fn from_env() -> APIAuth {
        let token = env::var("TOKEN").ok().expect("env missing TOKEN");
        let user = env::var("DEL_USER").ok().expect("env missing DEL_USER");
        APIAuth { user: user, token: token }
    }

    pub fn auth_headers(&self) -> (headers::ChefDeliveryUser,
                                   headers::ChefDeliveryToken) {
        (headers::ChefDeliveryUser(self.user.clone()),
         headers::ChefDeliveryToken(self.token.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::APIClient;
    use super::APIAuth;
    use std::env;

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
        fake_test_env();
        let client = APIClient::new_http("localhost:4343",
                                         "Chef");
        let url = client.api_url("foo/bar");
        assert_eq!("http://localhost:4343/api/v0/e/Chef/foo/bar", url)
    }

    #[test]
    fn https_api_url_test() {
        fake_test_env();
        let client = APIClient::new_https("localhost:4343",
                                          "Chef");
        let url = client.api_url("foo/bar");
        assert_eq!("https://localhost:4343/api/v0/e/Chef/foo/bar", url)
    }

    fn fake_test_env() {
        env::set_var("DEL_USER", "pete");
        env::set_var("TOKEN", "deadbeefcafe");
    }
    // #[test]
    // fn something() {
    //     let client = APIClient::new_https("172.31.6.130", "Chef");
    //     client.call();
    //     assert_eq!(1, 2);
    // }

    // #[test]
    // fn post_test() {
    //     let client = APIClient;
    //     let payme = "{\"a\":1}";
    //     let resp_body = client.post("localhost:4343", "foo/bra", payme);
    //     println!("result of post: {}", resp_body);
    //     assert_eq!("abc", resp_body);
    // }
}
