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

use cli::status::StatusClapOptions;
use command::Command;
use config::Config;
use http;
use serde_json;
use types::{DeliveryResult, ExitCode};
use utils::say::{say, sayln};
use std::time::Instant;
use json::server_status::*;

pub struct StatusCommand<'n> {
    pub options: &'n StatusClapOptions<'n>,
    pub config: &'n Config,
}

impl<'n> Command for StatusCommand<'n> {
    fn run(&self) -> DeliveryResult<ExitCode> {
        let client = try!(http::APIClient::from_config_with_basic_routing(&self.config));

        let start = Instant::now();
        let mut result = try!(client.get("api/_status"));
        let elapsed = start.elapsed();
        let elapsed_milli = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;

        let mut json_string = try!(http::APIClient::extract_pretty_json(&mut result));
        // Backwards compat: A few versions of the server shipped with a
        // key with a space in it.
        json_string = json_string.replace("configuration mode", "configuration_mode");

        if self.options.json {
            sayln("white", json_string.as_ref());
            return Ok(0)
        }

        // Closures to display status with the right color
        let say_status   = |s: &str| if s == "up" { say("green", s) } else { say("red", s) };
        let sayln_status = |s: &str| if s == "up" { sayln("green", s) } else { sayln("red", s) };

        // Replace "pong" with "up" because it is more human friendly word.
        json_string = json_string.replace("pong", "up");

        let s: ServerStatus = serde_json::from_str(&json_string)?;

        sayln("white", &format!(
                "Status information for Automate server {}...\n",
                self.config.api_host_and_port()?
        ));
        say("white", "Status: ");
        say_status(&s.status);

        if s.status == "up" {
            sayln("green", &format!(" (request took {} ms)", &elapsed_milli.to_string()));
        }

        sayln("white", &format!("Configuration Mode: {}", s.configuration_mode));

        // Backward compat: fips_mode was added later so it is an optional field.
        if let Some(fips) = s.fips_mode {
            let fips_output = if fips { "enabled" } else { "disabled" };
            sayln("white", &format!("FIPS Mode: {}", fips_output));
        }

        let ref u = s.upstreams[0];
        sayln("white", "Upstreams:");
        sayln("white", "  Lsyncd:");
        say("white", "    status: ");
        sayln_status(&u.lsyncd.status);
        sayln("white", "  PostgreSQL:");
        say("white",   "    status: ");
        sayln_status(&u.postgres.status);
        sayln("white", "  RabbitMQ:");
        say("white",   "    status: ");
        sayln_status(&u.rabbitmq.status);

        if let Some(ref node_health) = u.rabbitmq.node_health {
            sayln("white", "    node_health:");
            say("white",   "      status: ");
            sayln_status(&node_health.status);
        }

        if let Some(ref vhost_aliveness) = u.rabbitmq.vhost_aliveness {
            sayln("white", "    vhost_aliveness:");
            say("white",   "      status: ");
            sayln_status(&vhost_aliveness.status);
        }

        if let Some(fips) = s.fips_mode {
            if fips {
                let msg = "\nYour Automate Server is configured in FIPS mode.\n\
                    Please add the following to your cli.toml to enable Automate FIPS \
                    mode on your machine:\n\nfips = true\nfips_git_port = \"OPEN_PORT\"\n\n\
                    Replace OPEN_PORT with any port that is free on your machine.";
                sayln("white", msg);
            }
        }
        Ok(0)
    }
}
