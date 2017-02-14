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

use std::process::Command;
use std::env;
use errors::{DeliveryError, Kind};
use libc;
use utils::{path_to_string, generate_command_from_string};
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::error;

pub fn copy_recursive<P: ?Sized>(f: &P, t: &P) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    let from = f.as_ref();
    let to = t.as_ref();
    let result = try!(Command::new("cp")
         .arg("-R")
         .arg("-a")
         .arg(from.to_str().unwrap())
         .arg(to.to_str().unwrap())
         .output());
    super::cmd_success_or_err(&result, Kind::CopyFailed)
}

pub fn remove_recursive<P: ?Sized>(path: &P) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    try!(Command::new("rm")
         .arg("-rf")
         .arg(path.as_ref().to_str().unwrap())
         .output());
    Ok(())
}

pub fn chmod<P: ?Sized>(path: &P, setting: &str) -> Result<(), DeliveryError>
    where P: AsRef<Path>
{
    let result = try!(Command::new("chmod")
         .arg(setting)
         .arg(path.as_ref().to_str().unwrap())
         .output());
    super::cmd_success_or_err(&result, Kind::ChmodFailed)
}

pub fn chown_all<P: AsRef<Path>>(who: &str,
                                 paths: &[P]) -> Result<(), DeliveryError> {
    let mut command = Command::new("chown");
    command.arg("-R").arg(who);
    for p in paths {
        command.arg(&path_to_string(p));
    }
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => {
            return Err(DeliveryError{
                kind: Kind::FailedToExecute,
                detail: Some(format!("failed to execute chown: {}",
                                     error::Error::description(&e)))}) },
    };
    super::cmd_success_or_err(&output, Kind::ChmodFailed)
}

pub fn privileged_process() -> bool {
    match unsafe { libc::getuid() } {
        0 => true,
        _ => false
    }
}

// Abstraction for command creation. Needed because of how we're
// wrapping commands in Windows. See this function in the
// corresponding windows module.
pub fn make_command(cmd: &str) -> Command {
    Command::new(cmd)
}

/// Returns the absolute path for a given command, if it exists, by searching the `PATH`
/// environment variable.
///
/// If the command represents an absolute path, then the `PATH` seaching will not be performed.
/// If no absolute path can be found for the command, then `None` is returned.
pub fn find_command(command: &str) -> Option<PathBuf> {
    // If the command path is absolute and a file exists, then use that.
    let candidate = PathBuf::from(command);
    if candidate.is_absolute() && candidate.is_file() {
        return Some(candidate);
    }
    // Find the command by checking each entry in `PATH`. If we still can't find it,
    // give up and return `None`.
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let candidate = PathBuf::from(&path).join(command);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn copy_automate_nginx_cert(server: &str, port: &str) -> Result<String, DeliveryError>
{
    let cmd_str = format!("/opt/chefdk/embedded/bin/openssl s_client -connect {server}:{port} -showcerts", server=server, port=port);
    let mut command = try!(generate_command_from_string(&cmd_str));
    let result = try!(command.output());

    if !result.status.success() {
        Err(DeliveryError{
            kind: Kind::AutomateNginxCertFetchFailed,
            detail: Some(format!("STDOUT: {}\nSTDERR: {}",
                                 String::from_utf8_lossy(&result.stdout),
                                 String::from_utf8_lossy(&result.stderr)))
        })
    } else {
        let openssl_output = try!(String::from_utf8(result.stdout));
        match parse_certs_from_string(openssl_output) {
            None => Err(DeliveryError{
                kind: Kind::AutomateNginxCertFetchFailed,
                detail: Some(format!("The cert chain request to {server}:{port} was \
                                      successful but no certs were found. Have you set up \
                                      certificates for your Automate server?",
                                     server=server, port=port))
            }),
            Some(certs) => Ok(certs),
        }
    }
}

fn parse_certs_from_string(input: String) -> Option<String>
{
    let cert_split_on_begin: Vec<&str> = input.split("-----BEGIN CERTIFICATE-----\n").collect();
    if cert_split_on_begin.len() < 2 {
        return None
    }

    let cert_split_on_begin_minus_leading = &cert_split_on_begin[1..];
    let mut certs = String::new();
    for cert_block in cert_split_on_begin_minus_leading {
        let cert_trim_to_end: Vec<&str> = cert_block.split("-----END CERTIFICATE-----\n").collect();
        certs += &format!("-----BEGIN CERTIFICATE-----\n{}-----END CERTIFICATE-----\n",
                          &cert_trim_to_end[0]);
    }
    Some(certs)
}

#[test]
fn parse_certs_from_string_parses_multiple_certs_with_leading_middle_end_cruft() {
    let input = r#"leading
-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
middle content
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
trailing
"#;
    let expected = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
"#;
    let actual = parse_certs_from_string(input.to_string());
    assert_eq!(Some(expected.to_string()), actual);
}

#[test]
fn parse_certs_from_string_parses_multiple_certs_with_middle_end_cruft() {
    let input = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
middle content
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
trailing
"#;
    let expected = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
"#;
    let actual = parse_certs_from_string(input.to_string());
    assert_eq!(Some(expected.to_string()), actual);
}

#[test]
fn parse_certs_from_string_parses_multiple_certs_with_end_cruft() {
    let input = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
trailing
"#;
    let expected = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
"#;
    let actual = parse_certs_from_string(input.to_string());
    assert_eq!(Some(expected.to_string()), actual);
}

#[test]
fn parse_certs_from_string_parses_multiple_certs_with_no_cruft() {
    let input = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
"#;
    let expected = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
cert2
-----END CERTIFICATE-----
"#;
    let actual = parse_certs_from_string(input.to_string());
    assert_eq!(Some(expected.to_string()), actual);
}

#[test]
fn parse_certs_from_string_parses_single_cert_with_middle_end_cruft() {
    let input = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
middle content
trailing
"#;
    let expected = r#"-----BEGIN CERTIFICATE-----
cert1
-----END CERTIFICATE-----
"#;
    let actual = parse_certs_from_string(input.to_string());
    assert_eq!(Some(expected.to_string()), actual);
}

