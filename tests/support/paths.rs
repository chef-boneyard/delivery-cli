use std::env;
use std::path::PathBuf;
use delivery::utils::path_join_many::PathJoinMany;

pub fn exe_path() -> PathBuf {
    env::current_exe().unwrap()
}

pub fn root() -> PathBuf {
    let mut exe = exe_path(); // support
    exe.pop();                // tests/
    exe.pop();                // debug/
    exe.pop();                // target/
    exe.pop();                // delivery-cli/
    exe.join("tests")
}

pub fn fixtures() -> PathBuf {
    root().join_many(&["fixtures"])
}

pub fn fixture_file(names: &str) -> PathBuf {
    fixtures().join_many(&[names])
}

