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
use std::io::Write;
use errors::DeliveryError;

pub const STUNNEL_PATH: &'static str = "/opt/chefdk/embedded/bin/stunnel";

pub trait CheckFipsMode {
    fn is_fips_mode(&self) -> bool;
}

pub fn start_stunnel() -> Result<std::process::Child, DeliveryError> {
    let stunnel_config_path = try!(utils::home_dir(&[".chefdk/etc/stunnel.conf"])).to_str().unwrap().to_string();
    let mut stunnel_command = 
        try!(utils::generate_command_from_string(&format!("{stunnel_path} {config}",
                                                          stunnel_path=STUNNEL_PATH,
                                                          config=stunnel_config_path)));
    Ok(try!(stunnel_command.spawn()))

}

pub fn write_stunnel_cert_file(server: &str, api_port: &str) -> Result<(), DeliveryError> {
    let cert_string = try!(utils::copy_automate_nginx_cert(server, api_port));
    let mut cert_file =
        try!(File::create(try!(utils::home_dir(&[".chefdk/etc/automate-nginx-cert.pem"]))));
    try!(cert_file.write_all(cert_string.as_bytes()));
    Ok(())
}

pub fn generate_stunnel_config(server: &str, fips_git_port: &str) -> Result<(), DeliveryError> {
    try!(std::fs::create_dir_all(try!(utils::home_dir(&[".chefdk/etc/"]))));
    try!(std::fs::create_dir_all(try!(utils::home_dir(&[".chefdk/log/"]))));

    let stunnel_path = try!(utils::home_dir(&[".chefdk/etc/stunnel.conf"]));
    let mut conf_file = try!(File::create(&stunnel_path));
    try!(conf_file.write_all(b"fips = yes\n"));
    try!(conf_file.write_all(b"client = yes\n"));

    let output = "output = ".to_string();
    let output_conf = output + try!(utils::home_dir(&[".chefdk/log/stunnel.log\n"])).to_str().unwrap();
    try!(conf_file.write_all(output_conf.as_bytes()));

    try!(conf_file.write_all("foreground = quiet\n".as_bytes()));

    try!(conf_file.write_all(b"[git]\n"));

    let accept = "accept = ".to_string() + fips_git_port + "\n";
    try!(conf_file.write_all(accept.as_bytes()));

    let connect = "connect = ".to_string() + server + ":8989\n";
    try!(conf_file.write_all(connect.as_bytes()));

    let check_host = "checkHost = ".to_string() + server + "\n";
    try!(conf_file.write_all(check_host.as_bytes()));

    try!(conf_file.write_all(b"verifyChain = yes\n"));
    try!(conf_file.write_all(b"verify = 3\n"));

    let cert_location_pathbuf = try!(utils::home_dir(&[".chefdk/etc/automate-nginx-cert.pem\n"]));
    let cert_location = cert_location_pathbuf.to_str().unwrap();
    let ca_file = "CAfile = ".to_string() + cert_location;
    try!(conf_file.write_all(ca_file.as_bytes()));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;

    #[test]
    fn generate_stunnel_config_test() {
        let init = r#"fips = yes
client = yes
"#;
        let mut expected = init.to_string();
        expected += &format!("output = {}",
                             utils::home_dir(&[".chefdk/log/stunnel.log\n"]).unwrap().to_str().unwrap());
        expected += r#"foreground = quiet
[git]
accept = 36534
connect = automate.test:8989
checkHost = automate.test
verifyChain = yes
verify = 3
"#;
        expected += &format!("CAfile = {}",
                             utils::home_dir(&[".chefdk/etc/automate-nginx-cert.pem\n"]).unwrap().to_str().unwrap());
        generate_stunnel_config("automate.test", "36534").unwrap();
        let mut f = File::open(utils::home_dir(&[".chefdk/etc/stunnel.conf"]).unwrap()).unwrap();
        let mut actual = String::new();
        f.read_to_string(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }
}
