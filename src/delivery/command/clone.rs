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

use cli::clone::CloneClapOptions;
use command::Command;
use config::Config;
use errors::DeliveryError;
use errors::Kind::{CloneFailed, MissingSshPubKey, ProjectNotFound, UnauthorizedAction};
use fips;
use git;
use http::APIClient;
use std;
use types::{DeliveryResult, ExitCode};
use user::User;
use utils::say::{say, sayln};
use utils::{cwd, path_ext};

pub struct CloneCommand<'n> {
    pub options: &'n CloneClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for CloneCommand<'n> {
    fn setup(&self, child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
        if self.config.fips.unwrap_or(false) {
            try!(fips::setup_and_start_stunnel(&self.config, child_processes));
        }
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
            let msg = format!(
                "The destination path '{}' already exists.",
                project_root.to_str().unwrap()
            );
            return Err(DeliveryError::throw(CloneFailed, Some(msg)));
        }

        if let Some(e) = git::clone(self.options.project, &clone_url).err() {
            debug!("Raw Clone Error: {:?}", e);
            sayln(
                "red",
                &format!("Unable to clone project '{}'", self.options.project),
            );
            sayln("yellow", "Analyzing error:");

            // Verify that the user is well configured
            let user = User::load(&self.config, None)?;
            if !user.verify_pub_key() {
                let link = self.config.users_url()?;
                let msg = format!(
                    "The configured user '{}' does not have an ssh_pub_key.\
                     \nPlease login to the Automate server and configure your key \
                     at:\n\t{}",
                    self.config.user()?,
                    link
                );
                return Err(DeliveryError::throw(MissingSshPubKey, Some(msg)));
            }

            // Does the project exist?
            let o = self.config.organization()?;
            let p = self.options.project;
            if !APIClient::from_config(&self.config)?.project_exists(&o, p) {
                let msg = format!(
                    "You can find the list of available projects \
                     at:\n\t{}",
                    self.config.projects_url()?
                );
                return Err(DeliveryError::throw(
                    ProjectNotFound(p.to_string()),
                    Some(msg),
                ));
            }

            // Is the user powerful enough to perform this action?
            if let Some(ref d) = e.detail {
                if d.find("Unauthorized action").is_some() {
                    let link = self.config.users_url()?;
                    let msg = format!(
                        "Contact an administrator to grant you with appropriate \
                         permissions at the following url:\n\t{}",
                        link
                    );
                    return Err(DeliveryError::throw(UnauthorizedAction, Some(msg)));
                }
            }

            // We dont know what's the problem, throw the normal error
            return Err(e);
        }

        try!(git::update_delivery_remote(&delivery_url, &project_root));
        sayln("success", "Your project was cloned successfully.");

        // Should we automatically generate a cli.toml inside project?
        //
        // We could have the clone command to write the toml file inside
        // the project so that any other command that depends on the config
        // won't complain about it. But if we do that we need to consider:
        //
        // a) Should we notify the end-user that they have to add a
        //    `cli.toml` entry to their `.gitignore`?
        // b) Or should we just modify it automatically?
        //    (I wouldn't like this)
        // c) We could also add it to the chefdk generator.
        //
        // If we want to persue this option, uncomment the following lines:
        //try!(self.config.write_file(&project_root));
        //let gitignore = read_file(project_root.join(".gitignore"))?;
        //if gitignore.find("cli.toml").is_none() {
        //sayln("yellow", "Make sure you have a 'cli.toml' entry in your '.gitignore'");
        //}

        Ok(0)
    }
}
