//
// Copyright:: Copyright (c) 2017 Chef Software, Inc.
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

use types::DeliveryResult;
use http::APIClient;
use errors::DeliveryError;
use errors::Kind::UserNotFound;
use config::Config;
use serde_json;

#[derive(Deserialize, Debug)]
pub struct User {
  pub first: String,
  pub last: String,
  pub name: String,
  pub email: String,
  pub user_type: String,
  ssh_pub_key: String,
}

impl Default for User {
    fn default() -> Self {
        User {
          first: String::from(""),
          last: String::from(""),
          name: String::from(""),
          email: String::from(""),
          ssh_pub_key: String::from(""),
          user_type: String::from("internal"),
        }
    }
}

impl User {
    // Load a `User` from the specified `Config`, by default it will try to load
    // the user specified in the config but if a different user is provided, it will
    // load that user instead.
    //
    // # Example:
    //
    // ```
    // use delivery::user::User;
    //
    // let mine: User = User::load(&config, None);          <- Load user from config
    // let diff: User = User::load(&config, Some("link"));  <- Load user `link`
    // ```
    pub fn load(config: &Config, username: Option<&str>) -> DeliveryResult<Self> {
        let c_user = config.user()?;
        let name = username.unwrap_or(c_user.as_str());
        let client = APIClient::from_config(config)?;
        if !client.user_exists(&name) {
            return Err(DeliveryError::throw(UserNotFound(name.to_owned()), None))
        }
        let mut raw_json = client.get(&format!("users/{}", name))?;
        let json = APIClient::extract_pretty_json(&mut raw_json)?;
        let user: User = serde_json::from_str(&json)?;
        Ok(user)
    }

    pub fn verify_pub_key(&self) -> bool {
        !self.ssh_pub_key.is_empty()
    }

    pub fn set_ssh_pub_key(&mut self, key: &str) {
        self.ssh_pub_key = String::from(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_a_user_with_defaults() {
        let u = User::default();
        assert_eq!(u.first, "".to_string());
        assert_eq!(u.user_type, "internal".to_string());
    }

    #[test]
    fn user_verify_pub_key() {
        // ssh_pub_key is not set by default
        let mut u = User::default();
        assert!(!u.verify_pub_key());

        // Setting ssh_pub_key, and then verifying
        u.set_ssh_pub_key("SECRET");
        assert!(u.verify_pub_key());
    }
}
