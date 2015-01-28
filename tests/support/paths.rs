#![allow(unstable)]
use std::os;

pub fn exe_path() -> Path {
    os::self_exe_path().unwrap()
}

pub fn root() -> Path {
    let root_path = exe_path().join_many(&["..", "tests"]);
    os::make_absolute(&root_path).unwrap()
}

pub fn fixtures() -> Path {
    root().join_many(&["fixtures"])
}

pub fn fixture_file(names: &str) -> Path {
    fixtures().join_many(&[names])
}

