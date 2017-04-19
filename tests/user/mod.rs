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

extern crate delivery;
use mockito::{SERVER_ADDRESS, mock};
use delivery::config::Config;
use delivery::user::User;
use delivery::errors::Kind;
use tempdir::TempDir;
use std::env;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::Write;

fn setup() {
    let user = r#"
{
    "email": "link@zelda.io",
    "first": "Link",
    "last": "Hylian",
    "name": "link",
    "ssh_pub_key": "ssh-rsa SECRET",
    "user_type": "internal"
}
"#;
    mock("GET", "/api/v0/e/ent/users/link")
        .with_status(200)
        .match_header("Content-Type", "application/json")
        .with_body(&user)
        .create();
    mock("GET", "/api/v0/e/ent/users/ganon")
        .with_status(404)
        .create();
    mock("GET", "/api/v0/e/ent/users/bad")
        .with_status(200)
        .match_header("Content-Type", "application/json")
        .with_body("{\"name\": \"bad\"}")
        .create();
    mock("GET", "/api/v0/e/ent/orgs")
        .with_status(200)
        .match_header("Content-Type", "application/json")
        .with_body("{}")
        .create();
}

// This tests require to mock the api-tokens file, in order to do so we
// will generate a "temporal directory" and point the HOME Env variable
// to that directory, only for the specific calls we need to.
fn assert_user_with_mocked_home<F>(home: &Path, closure: F) where F: Fn() {
    let saved_home = env::var("HOME").expect("Missin HOME env var");
    env::set_var("HOME", home);
    closure();
    env::set_var("HOME", saved_home);
}

// Mocking api-tokens file and returning the temp directory
fn mock_api_tokens() -> TempDir {
    // Mock the api-tokens store
    let mock_tokens = "0.0.0.0:1234,ent,link|this_is_a_fake_token";
    let tmpdir = TempDir::new("mock-delivery-api-tokens").expect("Unable to create tmp dir");
    let delivery_dir = tmpdir.path().join(".delivery");
    let api_tokens_path = delivery_dir.join("api-tokens");
    fs::create_dir(&delivery_dir).expect("Unable to create .delivery dir");
    let mut api_tokens = File::create(&api_tokens_path).expect("Unable to create api-tokens file");
    api_tokens.write_all(mock_tokens.as_bytes()).expect("Unable to write api-tokens file");
    tmpdir
}

// Mocking the config that will point to mockito server address
fn mock_config() -> Config {
    let mut c = Config::default()
        .set_server(SERVER_ADDRESS)
        .set_api_protocol("http")
        .set_user("link")
        .set_enterprise("ent");
    c.saml = Some(false);
    c
}

test!(load_user_that_exist {
    let temp_home = mock_api_tokens();

    let user_closure = || {
        let user = User::load(&mock_config(), None);
        assert!(user.is_ok());
        let link = user.unwrap();
        assert_eq!(link.first, "Link");
    };

    assert_user_with_mocked_home(temp_home.path(), user_closure);
});

test!(loading_user_that_does_not_exist {
    let temp_home = mock_api_tokens();

    let user_closure = || {
        let user = User::load(&mock_config(), Some("ganon"));
        assert!(user.is_err());
        let ganon = user.unwrap_err();
        assert_eq!(ganon.detail, None);
        if let Kind::UserNotFound(u_name) = ganon.kind {
            assert_eq!(u_name, "ganon");
        } else {
            assert!(false, "Error kind mismatch");
        }
    };

    assert_user_with_mocked_home(temp_home.path(), user_closure);
});

test!(loading_user_with_bad_response_body {
    let temp_home = mock_api_tokens();

    let user_closure = || {
        let user = User::load(&mock_config(), Some("bad"));
        assert!(user.is_err());
        let error = user.unwrap_err();
        if let Kind::JsonParseError = error.kind {
            assert!(error.detail.is_some());
        } else {
            assert!(false, "Error kind mismatch");
        }
    };

    assert_user_with_mocked_home(temp_home.path(), user_closure);
});
