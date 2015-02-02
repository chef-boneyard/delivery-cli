#![allow(unstable)]
extern crate delivery;

use delivery::job::workspace::{Workspace};
use delivery::git::git_command;
use std::old_io::{self, TempDir, File};
use std::old_io::fs::PathExtensions;
use support::paths::fixture_file;
use std::old_io::process::Command;
use std::os;

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
        assert_eq!(p.stat().unwrap().perm, old_io::USER_RWX);
    }
});

fn copy_to_tmpdir(path: &str, tmpdir: &TempDir) {
    // This test will fail on windows!
    let cp_res = Command::new("cp")
        .arg("-R")
        .arg("-a")
        .arg(path)
        .arg(tmpdir.path().as_str().unwrap())
        .output().unwrap();
    if ! cp_res.status.success() {
        let output = String::from_utf8_lossy(cp_res.output.as_slice());
        let error = String::from_utf8_lossy(cp_res.error.as_slice());
        println!("{}\n{}\n{}", output.as_slice(), error.as_slice(), tmpdir.path().as_str().unwrap());
        panic!("Failed to copy data");
    }
}

// As reckless as I wanna be
#[allow(unused_must_use)]
fn setup_test_repo(hedge: &str) -> (TempDir, String) {
    let tmpdir = TempDir::new(hedge).unwrap();
    let test_repo_path = fixture_file("test_repo");
    copy_to_tmpdir(format!("{}/.delivery", test_repo_path.as_str().unwrap().as_slice()).as_slice(), &tmpdir);
    copy_to_tmpdir(format!("{}/README.md", test_repo_path.as_str().unwrap().as_slice()).as_slice(), &tmpdir);
    copy_to_tmpdir(format!("{}/cookbooks", test_repo_path.as_str().unwrap().as_slice()).as_slice(), &tmpdir);
    os::change_dir(tmpdir.path()).unwrap();
    git_command(&["init", tmpdir.path().as_str().unwrap()], tmpdir.path());
    git_command(&["add", "."], tmpdir.path()).unwrap();
    git_command(&["commit", "-a", "-m", "Initial Commit"], tmpdir.path()).unwrap();
    git_command(&["branch", "rust/test"], tmpdir.path()).unwrap();
    {
        let mut f = File::create(&tmpdir.path().join("freaky"));
        f.write(b"I like cookies");
    }
    git_command(&["add", "."], tmpdir.path()).unwrap();
    git_command(&["commit", "-a", "-m", "New file"], tmpdir.path()).unwrap();
    let commit_sha = git_command(&["rev-parse", "HEAD"], tmpdir.path()).unwrap();
    git_command(&["checkout", "master"], tmpdir.path()).unwrap();
    (tmpdir, commit_sha.stdout)
}

test!(setup_repo_for_change {
    let (test_repo, _) = setup_test_repo("setup_for_change");
    let tmpdir = TempDir::new("build-test").unwrap();
    let root = tmpdir.path().join("root");
    let ws = Workspace::new(&root);
    let _ = ws.build();
    let r = ws.setup_repo_for_change(
        test_repo.path().as_str().unwrap(),
        "rust/test",
        "master",
        ""
    );
    match r {
        Ok(()) => { },
        Err(e) => {
            panic!("{:?}", e);
        }
    }
    assert!(ws.repo.join(".delivery").is_dir());
    assert!(ws.repo.join("cookbooks").is_dir());
    assert!(ws.repo.join("freaky").is_file());
});

test!(setup_chef_for_job {
    let (test_repo, _) = setup_test_repo("chef_for_job");
    let tmpdir = TempDir::new("chef_job_test").unwrap();
    let root = tmpdir.path().join("root");
    let ws = Workspace::new(&root);
    panic_on_error!(ws.build());
    panic_on_error!(ws.setup_repo_for_change(
        test_repo.path().as_str().unwrap(),
        "rust/test",
        "master",
        ""
    ));
    panic_on_error!(ws.setup_chef_for_job());
    assert!(ws.chef.join("config.rb").is_file());
    assert!(ws.chef.join("dna.json").is_file());
    assert!(ws.chef.join_many(&["cookbooks", "delivery_test"]).is_dir());
    assert!(ws.chef.join_many(&["cookbooks", "build-essential"]).is_dir());
});

