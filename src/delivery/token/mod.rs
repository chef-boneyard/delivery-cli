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

//! Token Store
//!
//! Manage API tokens backed by a flat text file.
//!
//! The `TokenStore` manages a map of keys to tokens and a path to the
//! backing file. Adding or updating a token is done via `write_token`
//! and will immediately rewrite the backing file. Find an existing
//! token using `lookup`.
//!
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::collections::BTreeMap;
use std::env;
use errors::{DeliveryError, Kind};
use utils;
use utils::path_join_many::PathJoinMany;
use config::Config;
use http;
use utils::say::sayln;
use getpass;

#[derive(Debug)]
pub struct TokenStore {
    tokens: BTreeMap<String, String>,
    path: PathBuf
}

impl TokenStore {
    pub fn from_home() -> Result<TokenStore, DeliveryError> {
        let home_dot_delivery = match env::home_dir() {
            Some(home) => home.join_many(&[".delivery"]),
            None => {
                let msg = "unable to find home dir".to_string();
                return Err(DeliveryError{ kind: Kind::NoHomedir,
                                          detail: Some(msg) })
            }
        };
        try!(utils::mkdir_recursive(&home_dot_delivery));
        let token_path = home_dot_delivery.join_many(&["api-tokens"]);
        TokenStore::from_file(&token_path)
    }

    pub fn from_file(path: &PathBuf) -> Result<TokenStore, DeliveryError> {
        let tokens = try!(TokenStore::read_config(&path));
        let tstore = TokenStore {path: path.clone(), tokens: tokens};
        Ok(tstore)
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn lookup(&self,
                  server: &str, ent: &str, user: &str) -> Option<&String> {
        let key = TokenStore::key(server, ent, user);
        self.tokens.get(&key)
    }

    pub fn write_token(&mut self,
                       server: &str,
                       ent: &str,
                       user: &str,
                       token: &str) -> Result<Option<String>, DeliveryError> {

        let result = self.set_token(server, ent, user, token);
        match self.write_config() {
            Ok(_) => Ok(result),
            Err(e) => Err(e)
        }
    }

    pub fn request_token(config: &Config) -> Result<String, DeliveryError>  {
      let ent = try!(config.enterprise());
      let user = try!(config.user());
      let api_server = try!(config.api_host_and_port());
      let mut tstore = try!(TokenStore::from_home());
      let pass = getpass::read("Delivery password: ");
      let token = try!(http::token::request(&config, &pass));
      sayln("magenta", &format!("token: {}", &token));
      try!(tstore.write_token(&api_server, &ent, &user, &token));
      sayln("green", &format!("saved API token to: {}", tstore.path().display()));
      Ok(token)
    }

    fn key(server: &str, ent: &str, user: &str) -> String {
        format!("{},{},{}", server, ent, user)
    }

    fn set_token(&mut self,
                 server: &str, ent: &str, user: &str,
                 token: &str) -> Option<String> {
        let key = TokenStore::key(server, ent, user);
        self.tokens.insert(key, token.to_string())
    }

    fn write_config(&self) -> Result<(), DeliveryError> {
        let mut file = try!(File::create(&self.path));
        for (k, v) in self.tokens.iter() {
            let line = format!("{}|{}\n", k, v);
            try!(file.write_all(line.as_bytes()));
        }
        Ok(())
    }

    fn read_config(path: &PathBuf) -> Result<BTreeMap<String, String>, DeliveryError> {
        let mut opener = OpenOptions::new();
        opener.create(true);
        opener.truncate(false);
        opener.write(false);
        opener.read(true);
        let file = try!(opener.open(&path));
        let reader = BufReader::new(file);
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        for line in reader.lines() {
            let real_line = try!(line);
            let split = real_line.trim().split("|");
            let items = split.collect::<Vec<&str>>();
            if items.len() == 2 {
                let key = items[0].to_string();
                let token = items[1].to_string();
                map.insert(key, token);
            } else {
                println!("skipping malformed line: {}", real_line);
            }
        }
        Ok(map)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::File;
    use tempdir::TempDir;
    use utils::path_join_many::PathJoinMany;

     #[test]
    fn create_from_empty_test() {
        let tempdir = TempDir::new("t1").ok().expect("TempDir failed");
        let path = tempdir.path();
        let tfile = path.join_many(&["api-tokens"]);
        println!("dbg tfile: {:?}", tfile);
        let mut tstore = TokenStore::from_file(&tfile).ok().expect("no create");
        println!("got: {:?}", tstore);
        assert_eq!(None, tstore.lookup("127.0.0.1", "acme", "bob"));
        let write_result = tstore.write_token("127.0.0.1", "acme", "bob",
                                              "beefbeef");
        assert_eq!(true, write_result.is_ok());
        assert_eq!(&"beefbeef", tstore.lookup("127.0.0.1",
                                             "acme", "bob").unwrap());
        // why doesn't this work in this context?
        // let mut f = try!(File::open(&tfile));
        let mut f = File::open(&tfile).ok().expect("tfile open error");
        let mut content = String::new();
        assert_eq!(true, f.read_to_string(&mut content).is_ok());
        assert_eq!("127.0.0.1,acme,bob|beefbeef\n", content);
    }
}
