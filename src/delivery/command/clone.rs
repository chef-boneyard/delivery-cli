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

use std;
use fips;
use git;
use cli::clone::CloneClapOptions;
use types::{DeliveryResult, ExitCode};
use errors::DeliveryError;
use errors::Kind::{MissingSshPubKey, CloneFailed, ProjectNotFound, UnauthorizedAction};
use utils::say::{say, sayln};
use utils::cwd;
use utils::path_ext;
use http::APIClient;
use user::User;
use command::Command;
use config::Config;

pub struct CloneCommand<'n> {
    pub options: &'n CloneClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for CloneCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        try!(fips::setup_and_start_stunnel_if_fips_mode(&self.config, child_processes));
        Ok(())
    }

    fn run(&self) -> DeliveryResult<ExitCode> {
        sayln("green", "Chef Delivery");
        say("white", "Cloning ");
        let delivery_url = try!(self.config.delivery_git_ssh_url());
        let project_root = cwd().join(self.options.project);
        let clone_url = if self.options.git_url.is_empty() {
            delivery_url.clone()
        } else {
            String::from(self.options.git_url)
        };

        say("yellow", &clone_url);
        say("white", " to ");
        sayln("magenta", &format!("{}", self.options.project));

        // Verify if the destination path already exist.
        if path_ext::is_dir(&project_root) {
            let msg  = format!("The destination path '{}' already exists.",
                               project_root.to_str().unwrap());
            return Err(DeliveryError::throw(CloneFailed, Some(msg)))
        }

        if let Some(e) = git::clone(self.options.project, &clone_url).err() {
            debug!("Raw Clone Error: {:?}", e);
            sayln("red", &format!("Unable to clone project '{}'", self.options.project));
            sayln("yellow", "Analyzing reason of failure...");

            // Verify that the user is well configured
            let user = User::load(&self.config, None)?;
            if !user.verify_pub_key() {
                let link = self.config.users_url()?;
                let msg  = format!("The configured user '{}' does not have an ssh_pub_key.\
                            \nPlease login to the Automate server and configure your key \
                            at:\n\t{}", self.config.user()?, link);
                return Err(DeliveryError::throw(MissingSshPubKey, Some(msg)))
            }

            // Does the project exist?
            let o = self.config.organization()?;
            let p = self.options.project;
            if !APIClient::from_config(&self.config)?.project_exists(&o, p) {
                let msg = format!("You can find the list of available projects \
                                  at:\n\t{}", self.config.projects_url()?);
                return Err(
                    DeliveryError::throw(ProjectNotFound(p.to_string()), Some(msg))
                )
            }

            // Is the user powerful enough to perform this action?
            if let Some(ref d) = e.detail {
                if d.find("Unauthorized action").is_some() {
                let link = self.config.users_url()?;
                let msg = format!("Contact an administrator to grant you with appropriate \
                           permissions at the following url:\n\t{}", link);
                    return Err(DeliveryError::throw(UnauthorizedAction, Some(msg)))
                }
            }

            // We dont know what's the problem, throw the normal error
            return Err(e)
        }

        try!(git::create_or_update_delivery_remote(&delivery_url,
                                                   &project_root));
        Ok(0)
    }
}
