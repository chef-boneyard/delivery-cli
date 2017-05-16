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

use std;
use utils;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Write;
use errors::DeliveryError;
use types::DeliveryResult;
use errors::Kind;
use config::Config;

pub fn setup_and_start_stunnel(config: &Config,
                               child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
    if !Path::new(&stunnel_path()).exists() {
        return Err(DeliveryError{
            kind: Kind::FipsNotSupportedForChefDKPlatform,
            detail: None
        })
    }

    let server = validate!(config, server);
    let fips_git_port = validate!(config, fips_git_port);
    try!(generate_stunnel_config(&try!(stunnel_config_path()), &server, &fips_git_port,
                                 config.fips_custom_cert_filename.as_ref()));
    try!(start_stunnel(child_processes));
    Ok(())
}

pub fn merge_fips_options_and_config(fips: bool, fips_git_port: &str, fips_custom_cert_filename: &str,
                                     mut config: Config) -> DeliveryResult<Config> {
    if config.fips.is_none() {
        config.fips = Some(fips);
    }

    // Only override config default with option if user specfied something
    // other than the default of empty string.
    if fips_custom_cert_filename.len() > 0 {
        config.fips_custom_cert_filename = Some(String::from(fips_custom_cert_filename));
    }

    let new_config = config.set_fips_git_port(fips_git_port);
    Ok(new_config)
}

fn start_stunnel(child_processes: &mut Vec<std::process::Child>) -> DeliveryResult<()> {
    // On windows, stunnel behaves very differently, so we need to run it as a service,
    // instead of starting and stopping as a child process via rust as we do in unix.
    if cfg!(target_os = "windows") {
        try!(utils::generate_command_from_string(&format!("{stunnel_path} -install -quiet",
                                                          stunnel_path=stunnel_path())).output());

        try!(utils::generate_command_from_string(&format!("{stunnel_path} -start -quiet",
                                                          stunnel_path=stunnel_path())).output());

        try!(utils::generate_command_from_string(&format!("{stunnel_path} -reload -quiet",
                                                          stunnel_path=stunnel_path())).output());

    } else {
        let unix_stunnel_config_path = try!(stunnel_config_path()).to_str().unwrap().to_string();
        let mut stunnel_command =
            utils::generate_command_from_string(&format!("{stunnel_path} {config}",
                                                         stunnel_path=stunnel_path(),
                                                         config=unix_stunnel_config_path)
            );
        child_processes.push(try!(stunnel_command.spawn()));
    };


    Ok(())
}

fn stunnel_config_path() -> Result<PathBuf, DeliveryError> {
    if cfg!(target_os = "windows") {
        Ok(PathBuf::from("C:\\opscode\\chefdk\\embedded\\stunnel.conf"))
    } else {
        utils::home_dir(&[".chefdk/etc/stunnel.conf"])
    }
}

fn stunnel_path() -> String {
    if cfg!(target_os = "windows") {
        String::from("C:\\opscode\\chefdk\\embedded\\bin\\stunnel.exe")
    } else {
        String::from("/opt/chefdk/embedded/bin/stunnel")
    }
}

fn generate_stunnel_config<P, T>(stunnel_config_path: P,
                           server: &str, fips_git_port: &str,
                           fips_custom_cert_filename: Option<T>) -> Result<(), DeliveryError>
            where P: AsRef<Path>,
                  T: AsRef<str> {
    try!(std::fs::create_dir_all(try!(utils::home_dir(&[".chefdk/etc/"]))));
    try!(std::fs::create_dir_all(try!(utils::home_dir(&[".chefdk/log/"]))));

    let newline_str = if cfg!(target_os = "windows") { "\r\n" } else { "\n" };

    let mut conf_file = try!(File::create(&stunnel_config_path));

    let fips = "fips = yes".to_string() + newline_str;
    try!(conf_file.write_all(fips.as_bytes()));

    let client = "client = yes".to_string() + newline_str;
    try!(conf_file.write_all(client.as_bytes()));

    let output = "output = ".to_string();
    let output_conf = output + try!(utils::home_dir(&[".chefdk/log/stunnel.log"])).to_str().unwrap() + newline_str;
    try!(conf_file.write_all(output_conf.as_bytes()));

    if !cfg!(target_os = "windows") {
        try!(conf_file.write_all(b"foreground = quiet\n"))
    }

    let git = "[git]".to_string() + newline_str;
    try!(conf_file.write_all(git.as_bytes()));

    let accept = "accept = ".to_string() + fips_git_port + newline_str;
    try!(conf_file.write_all(accept.as_bytes()));

    let connect = "connect = ".to_string() + server + ":8989" + newline_str;
    try!(conf_file.write_all(connect.as_bytes()));

    let check_host = "checkHost = ".to_string() + server + newline_str;
    try!(conf_file.write_all(check_host.as_bytes()));

    let verify_chain = "verifyChain = yes".to_string() + newline_str;
    try!(conf_file.write_all(verify_chain.as_bytes()));

    let verify = "verify = 3".to_string() + newline_str;
    try!(conf_file.write_all(verify.as_bytes()));

    if let Some(cert_str) = fips_custom_cert_filename {
        let ca_file = "CAfile = ".to_string() + cert_str.as_ref() + newline_str;
        try!(conf_file.write_all(ca_file.as_bytes()));
    } else {
        let ca_file = "CAfile = ".to_string() + &utils::ca_path() + newline_str;
        try!(conf_file.write_all(ca_file.as_bytes()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;

    #[test]
    fn generate_stunnel_config_when_ca_is_none_test() {
        let init = r#"fips = yes
client = yes
"#;
        let mut expected = init.to_string();
        expected += &format!("output = {}",
                             utils::home_dir(&[".chefdk/log/stunnel.log\n"]).unwrap().to_str().unwrap());
        if !cfg!(target_os = "windows") {
            expected += "foreground = quiet\n";
        }
        expected += r#"[git]
accept = 36534
connect = automate.test:8989
checkHost = automate.test
verifyChain = yes
verify = 3
"#;
        expected += &format!("CAfile = {}\n",
                             utils::ca_path());
        let temp_conf_path = PathBuf::from("/tmp/test1.conf");
        let custom_cert: Option<String> = None;
        generate_stunnel_config(&temp_conf_path, "automate.test", "36534", custom_cert).unwrap();
        let mut f = File::open(temp_conf_path).unwrap();
        let mut actual = String::new();
        f.read_to_string(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn generate_stunnel_config_when_ca_is_some_test() {
        let ca_path = "/my/fun/ca.crt";

        let init = r#"fips = yes
client = yes
"#;
        let mut expected = init.to_string();
        expected += &format!("output = {}",
                             utils::home_dir(&[".chefdk/log/stunnel.log\n"]).unwrap().to_str().unwrap());
        if !cfg!(target_os = "windows") {
            expected += "foreground = quiet\n";
        }
        expected += r#"[git]
accept = 36534
connect = automate.test:8989
checkHost = automate.test
verifyChain = yes
verify = 3
"#;
        expected += &format!("CAfile = {}\n",
                             ca_path);
        let temp_conf_path = PathBuf::from("/tmp/test2.conf");
        generate_stunnel_config(&temp_conf_path, "automate.test", "36534",
                                Some(String::from(ca_path))).unwrap();
        let mut f = File::open(temp_conf_path).unwrap();
        let mut actual = String::new();
        f.read_to_string(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }
}
