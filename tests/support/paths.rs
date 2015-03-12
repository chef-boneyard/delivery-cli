use std::env;
use std::path::PathBuf;
use delivery::utils::path_join_many::PathJoinMany;

pub fn exe_path() -> PathBuf {
    env::current_exe().unwrap()
}

pub fn root() -> PathBuf {
    exe_path().parent().unwrap().parent().unwrap().parent().unwrap().join("tests")
}

pub fn fixtures() -> PathBuf {
    root().join_many(&["fixtures"])
}

pub fn fixture_file(names: &str) -> PathBuf {
    fixtures().join_many(&[names])
}

