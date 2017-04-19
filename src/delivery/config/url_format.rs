//
// Author: Salim Afiune (afiune@chef.io)
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

// This module implements helper fn to format useful urls that we use to
// give meaningful messages to our users of where to go in the UI.

use config::Config;
use types::DeliveryResult;

impl Config {

    // Users url
    //
    // The url we use to manage users. (ssh-pub-key, permissions, etc.)
    pub fn users_url(&self) -> DeliveryResult<String> {
        let p = self.api_protocol()?;
        let s = self.server()?;
        let e = self.enterprise()?;
        Ok(format!("{}://{}/e/{}/#/users", p, s, e))
    }

    // Organizations url
    //
    // List of organizations within an enterprise.
    pub fn organizations_url(&self) -> DeliveryResult<String> {
        let p = self.api_protocol()?;
        let s = self.server()?;
        let e = self.enterprise()?;
        Ok(format!("{}://{}/e/{}/#/organizations", p, s, e))
    }

    // Projects url
    //
    // List of projects within an organization.
    pub fn projects_url(&self) -> DeliveryResult<String> {
        let p = self.api_protocol()?;
        let s = self.server()?;
        let e = self.enterprise()?;
        let o = self.organization()?;
        Ok(format!("{}://{}/e/{}/#/organizations/{}", p, s, e, o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_users_url() {
        let mut conf    = Config::default();
        conf.server     = Some("automate.example.com".to_string());
        conf.enterprise = Some("test".to_string());
        assert_eq!("https://automate.example.com/e/test/#/users".to_string(),
                   conf.users_url().unwrap());
    }

    #[test]
    fn test_organizations_url() {
        let mut conf    = Config::default();
        conf.server     = Some("automate.example.com".to_string());
        conf.enterprise = Some("test".to_string());
        assert_eq!("https://automate.example.com/e/test/#/organizations".to_string(),
                   conf.organizations_url().unwrap());
    }

    #[test]
    fn test_projects_url() {
        let mut conf      = Config::default();
        conf.server       = Some("server".to_string());
        conf.enterprise   = Some("test".to_string());
        conf.organization = Some("org".to_string());
        assert_eq!("https://server/e/test/#/organizations/org".to_string(),
                   conf.projects_url().unwrap());
    }
}
