extern crate delivery;

use delivery::job::workspace::{Workspace};
use delivery::git::git_command;
use delivery::utils::copy_recursive;
use std::old_io::{self, TempDir, File};
use std::old_io::fs::PathExtensions;
use support::paths::fixture_file;
use std::old_io::process::Command;
use std::env;

fn setup() { }

test!(build {
    let tmpdir = TempDir::new("build-test").unwrap();
    let root = tmpdir.path().join("root");
    let ws = Workspace::new(&root);
    ws.build().unwrap();
    for p in [
      &ws.root,
      &ws.chef,
      &ws.cache,
      &ws.repo
    ].iter() {
        assert!(p.is_dir());
    }
});
