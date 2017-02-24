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
use utils::say::{say, sayln, ERROR_COLOR, SUCCESS_COLOR};
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
            println!("{}", json_string);
            return Ok(0)
        }

        // Replace "pong" with "up" because it is more human friendly word.
        json_string = json_string.replace("pong", "up");

        let s: ServerStatus = serde_json::from_str(&json_string)?;

        println!("{}", &format!("Status information for Automate server {}...\n",
                                try!(self.config.api_host_and_port())));
        print!("Status: ");
        print_status(&s.status, false, self.options.no_color);
        if s.status == "up" {
            if self.options.no_color {
                println!("{}", &format!(" ({} ms)", &elapsed_milli.to_string()));
            } else {
                sayln(SUCCESS_COLOR, &format!(" ({} ms)", &elapsed_milli.to_string()));
            }
        }

        println!("Configuration Mode: {}", s.configuration_mode);

        // Backward compat: fips_mode was added later so it is an optional field.
        if let Some(fips) = s.fips_mode {
            let fips_output = if fips { "enabled" } else { "disabled" };
            println!("FIPS Mode: {}", fips_output);
        }

        println!("Upstreams:");
        println!("  Lsyncd:");
        print!(  "    status: ");
        print_status(&s.upstreams[0].lsyncd.status, true, self.options.no_color);
        println!("  PostgreSQL:");
        print!(  "    status:");
        print_status(&s.upstreams[0].postgres.status, true, self.options.no_color);
        println!("  RabbitMQ:");
        print!(  "    status: ");
        print_status(&s.upstreams[0].rabbitmq.status, true, self.options.no_color);

        if let Some(ref node_health) = s.upstreams[0].rabbitmq.node_health {
            println!("    node_health:");
            print!(  "      status: ");
            print_status(&node_health.status, true, self.options.no_color);
        }

        if let Some(ref vhost_aliveness) = s.upstreams[0].rabbitmq.vhost_aliveness {
            println!("    vhost_aliveness:");
            print!(  "      status: ");
            print_status(&vhost_aliveness.status, true, self.options.no_color);
        }

        if let Some(fips) = s.fips_mode {
            if fips {
                println!("\nYour Automate Server is configured in FIPS mode.");
                println!("Please add the following to your cli.toml to enable Automate FIPS mode on your machine:\n");
                println!("fips = true");
                println!("fips_git_port = OPEN_PORT\n");
                println!("Replace OPEN_PORT with any port that is free on your machine.");
            }
        }

        Ok(0)
    }
}

fn print_status(status: &str, newline: bool, no_color_mode: bool) {
    if no_color_mode {
        if newline {
            println!("{}", status);
        } else {
            print!("{}", status);
        }
    } else {
        let color = if status == "up" { SUCCESS_COLOR } else { ERROR_COLOR };
        if newline {
            sayln(color, status);
        } else {
            say(color, status);
        }
    }
}
