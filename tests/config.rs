extern crate delivery;
extern crate log;

use delivery::config::{Config};
use support::paths::fixture_file;
use std::path::PathBuf;

fn setup() { }

fn _load_test_config_file() -> Config {
    let config_path = fixture_file("config");
    Config::load_config(&config_path).unwrap()
}

test!(load_config {
    let config = _load_test_config_file();
    assert_eq!(config.server, Some("127.0.0.1".to_string()));
});

test!(load_config_returns_defaults_on_failure {
    // I suppose someone might have a bogonista, and that would make this
    // test unstable. Maybe those people are good people, maybe they are
    // bad people. I do not judge. But I use the path anyway.
    let config = Config::load_config(&PathBuf::from("/bogonista")).unwrap();
    assert_eq!(config.server, None);
    assert_eq!(config.enterprise, None);
    assert_eq!(config.organization, None);
    assert_eq!(config.project, None);
    assert_eq!(config.user, None);
    assert_eq!(config.git_port, Some("8989".to_string()));
});

