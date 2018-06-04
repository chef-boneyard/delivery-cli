use delivery::git::git_command;
use delivery::utils::copy_recursive;
use delivery::utils::path_join_many::PathJoinMany;
use delivery::utils::say;
use serde_json;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Output};
use support::paths::fixture_file;
use tempdir::TempDir;

// ** Functions used in tests **

macro_rules! setup {
    () => {
        say::turn_off_spinner();
    };
}

/// Sets up a mock delivery git project from the test_repo fixture.
/// Includes copying in the `.delivery/config.json` that you plan
/// on using.
fn setup_mock_delivery_project_git(dot_config: &str) -> TempDir {
    let tmpdir = TempDir::new("mock-delivery-remote").unwrap();
    let test_repo_path = fixture_file("test_repo");
    panic_on_error!(copy_recursive(
        &test_repo_path.join(".delivery"),
        &tmpdir.path().to_path_buf()
    ));
    panic_on_error!(copy_recursive(
        &test_repo_path.join("README.md"),
        &tmpdir.path().to_path_buf()
    ));
    panic_on_error!(copy_recursive(
        &test_repo_path.join("cookbooks"),
        &tmpdir.path().to_path_buf()
    ));
    panic_on_error!(copy_recursive(
        &fixture_file(dot_config),
        &tmpdir.path().join_many(&[".delivery", "config.json"])
    ));
    panic_on_error!(git_command(
        &["init", tmpdir.path().to_str().unwrap()],
        tmpdir.path()
    ));
    panic_on_error!(git_command(&["add", "."], tmpdir.path()));
    panic_on_error!(git_command(
        &["commit", "-a", "-m", "Initial Commit"],
        tmpdir.path()
    ));
    tmpdir
}

/// Given a path, it copies the build cookbook into it, and turns it
/// into a git repository
fn setup_build_cookbook_project(tmpdir: &Path) {
    let build_cookbook_path = fixture_file("delivery_test");
    panic_on_error!(copy_recursive(&build_cookbook_path, &tmpdir.to_path_buf()));
    panic_on_error!(git_command(
        &["init", tmpdir.join("delivery_test").to_str().unwrap()],
        &tmpdir.join("delivery_test")
    ));
    panic_on_error!(git_command(&["add", "."], &tmpdir.join("delivery_test")));
    panic_on_error!(git_command(
        &["commit", "-a", "-m", "Initial Commit"],
        &tmpdir.join("delivery_test")
    ));
}

/// Clones a mock delivery git project to a local copy, as if it was
/// on a workstation.
fn setup_local_project_clone(delivery_project_git: &Path) -> TempDir {
    let tmpdir = TempDir::new("local-project").unwrap();
    panic_on_error!(git_command(
        &[
            "clone",
            delivery_project_git.to_str().unwrap(),
            tmpdir.path().to_str().unwrap()
        ],
        tmpdir.path()
    ));
    let mut command = delivery_cmd();
    command
        .arg("setup")
        .arg("--user")
        .arg("cavalera")
        .arg("--server")
        .arg("localhost")
        .arg("--ent")
        .arg("family")
        .arg("--org")
        .arg("sepultura")
        .arg("--for")
        .arg("master")
        .arg("--config-path")
        .arg(tmpdir.path().to_str().unwrap());
    assert_command_successful(&mut command, &tmpdir.path());
    tmpdir
}

/// Makes a change to a project on the named branch. Creates a
/// file named `filename` and writes some stuff to it.
///
/// When it returns, the project you pass in will be left on your
/// new branch.
fn setup_change(tmpdir: &Path, branch: &str, filename: &str) {
    panic_on_error!(git_command(&["checkout", "master"], tmpdir));
    panic_on_error!(git_command(&["branch", branch], tmpdir));
    {
        let mut f = panic_on_error!(File::create(&tmpdir.join(filename)));
        panic_on_error!(f.write_all(b"I like cookies"));
    }
    panic_on_error!(git_command(&["add", filename], tmpdir));
    panic_on_error!(git_command(&["commit", "-a", "-m", filename], tmpdir));
}

/// Runs the `Command` in `Dir`, and makes sure it exists with 0
/// Returns the result
fn assert_command_successful(command: &mut Command, dir: &Path) -> Output {
    let result = panic_on_error!(command.current_dir(&dir).output());
    if !result.status.success() {
        let output = String::from_utf8_lossy(&result.stdout);
        let error = String::from_utf8_lossy(&result.stderr);
        panic!(
            "Failed command {:?}\nOUT: {}\nERR: {}\nPath: {}",
            command,
            &output,
            &error,
            dir.to_str().unwrap()
        );
    };
    result
}

/// Same as `assert_command_successful` above, excepts it asserts the command
/// exits with a non-0 status code
fn assert_command_failed(command: &mut Command, dir: &Path) -> Output {
    let result = panic_on_error!(command.current_dir(&dir).output());
    if result.status.success() {
        let output = String::from_utf8_lossy(&result.stdout);
        let error = String::from_utf8_lossy(&result.stderr);
        panic!(
            "Command {:?} should have failed!\nOUT: {}\nERR: {}\nPath: {}",
            command,
            &output,
            &error,
            dir.to_str().unwrap()
        );
    };
    result
}

/// Builds the command to run delivery review
fn delivery_review_command(pipeline: &str) -> Command {
    let mut command = delivery_cmd();
    command
        .arg("review")
        .arg("--no-open")
        .arg("--for")
        .arg(pipeline);
    command
}

/// Builds the command to run a sample job
fn delivery_verify_command(job_root: &Path) -> Command {
    let mut command = delivery_cmd();
    command
        .arg("job")
        .arg("verify")
        .arg("lint")
        .arg("--no-spinner")
        .arg("--job-root")
        .arg(job_root.to_str().unwrap());
    command
}

/// Returns a Command set to the delivery binary created when you
/// ran `cargo test`.
fn delivery_cmd() -> Command {
    let mut delivery_path = env::current_exe().unwrap();
    delivery_path.pop();
    delivery_path.pop();
    Command::new(delivery_path.join("delivery").to_str().unwrap())
}

/// A handy debugging function. Insert it when you want to sleep,
/// pass it a tmpdir, and you can inspect it.
///
/// Make sure you run `cargo test -- --nocapture` to see the output.
#[allow(dead_code)]
fn debug_sleep(tmpdir: &TempDir) {
    println!("Sleeping for 1000 seconds for {:?}", tmpdir.path());
    panic_on_error!(Command::new("sleep").arg("1000").output());
}

// ** Actual tests **

test!(review_with_an_invalid_config {
    let delivery_project_git = setup_mock_delivery_project_git("invalid_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_review_command("rust/test");
    assert_command_failed(&mut command, &local_project.path());
});

test!(job_verify_lint_with_path_config {
    let delivery_project_git = setup_mock_delivery_project_git("path_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_successful(&mut command, &local_project.path());
});

test!(job_verify_lint_with_git_config {
    let delivery_project_git = setup_mock_delivery_project_git("git_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_build_cookbook_project(&job_root.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_successful(&mut command, &local_project.path());
});

// TODO: This test requires internet access... we should move it out
// into an Acceptance-stage test instead of here in lint tests. It's
// impossible to run on a plane, for instance :(
test!(job_verify_lint_with_public_supermarket_config {
    let delivery_project_git = setup_mock_delivery_project_git("public_supermarket_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_failed(&mut command, &local_project.path());
    assert!(job_root.path().join_many(&["chef", "cookbooks", "httpd"]).is_dir());
    assert!(job_root.path().join_many(&["chef", "cookbooks", "httpd", "templates", "default", "magic.erb"]).is_file());
});

// TODO: This test requires internet access...
// We are mocking that we are passing a Private Supermarket but instead we
// will use the public one (verify the mock json file) perhaps we can customize
// this in acceptance::functional and have an oficial private supermarket
test!(job_verify_lint_with_private_supermarket_config {
   let delivery_project_git = setup_mock_delivery_project_git("private_supermarket_config.json");
   let local_project = setup_local_project_clone(&delivery_project_git.path());
   let job_root = TempDir::new("job-root").unwrap();
   setup_change(&local_project.path(), "rust/test", "freaky");
   let mut command = delivery_verify_command(&job_root.path());
   assert_command_successful(&mut command, &local_project.path());
   assert!(job_root.path().join_many(&["chef", "cookbooks", "delivery-truck"]).is_dir());
   assert!(job_root.path().join_many(&["chef", "cookbooks", "delivery-truck", "recipes", "lint.rb"]).is_file());
});

test!(job_verify_dna_json {
    let delivery_project_git = setup_mock_delivery_project_git("path_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_successful(&mut command, &local_project.path());
    let mut dna_file = panic_on_error!(File::open(&job_root.path().join_many(&["chef", "dna.json"])));
    let mut dna_json = String::new();
    panic_on_error!(dna_file.read_to_string(&mut dna_json));
    let dna_data: Value = panic_on_error!(serde_json::from_str(&dna_json));
    match dna_data.pointer("/delivery/workspace/repo") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), job_root.path().join("repo").to_str().unwrap());
        },
        None => panic!("No delivery/workspace/repo, {}", dna_data)
    };
    match dna_data.pointer("/delivery/workspace/chef") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), job_root.path().join("chef").to_str().unwrap());
        },
        None => panic!("No delivery/workspace/chef, {}", dna_data)
    };
    match dna_data.pointer("/delivery/workspace/cache") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), job_root.path().join("cache").to_str().unwrap());
        },
        None => panic!("No delivery/workspace/cache, {}", dna_data)
    };
    match dna_data.pointer("/delivery/workspace/root") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), job_root.path().to_str().unwrap());
        },
        None => panic!("No delivery/workspace/root, {}", dna_data)
    };
    match dna_data.pointer("/delivery_builder/build_user") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), "dbuild");
        },
        None => panic!("No delivery_builderl/build_user, {}", dna_data)
    };
});

// This test is verifying that, when a project has custom attributes inside
// the `.delivery/config.json` they are available for the build_cookbook on
// every phase of the pipeline.
test!(job_verify_dna_json_with_extra_attributes {
    let delivery_project_git = setup_mock_delivery_project_git("extra_attributes_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_build_cookbook_project(&job_root.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_successful(&mut command, &local_project.path());
    let mut dna_file = panic_on_error!(File::open(&job_root.path().join_many(&["chef", "dna.json"])));
    let mut dna_json = String::new();
    panic_on_error!(dna_file.read_to_string(&mut dna_json));
    let dna_data: Value = panic_on_error!(serde_json::from_str(&dna_json));
    match dna_data.pointer("/delivery/config/build_cookbook") {
        Some(data) => {
            assert!(data.is_object());
            assert_eq!(
                data.as_object().unwrap().get("name").unwrap(),
               "delivery_test"
            );
        },
        None => panic!("No delivery/config/build_cookbook, {}", dna_data)
    };
    match dna_data.pointer("/delivery/config/attributes/key") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), "data");
        },
        None => panic!("No delivery/config/attributes/key, {}", dna_data)
    };
    match dna_data.pointer("/delivery/config/unlimited") {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_str().unwrap(), "configurable_data");
        },
        None => panic!("No delivery/config/unlimited, {}", dna_data)
    };
    match dna_data.pointer("/delivery/config/more") {
        Some(data) => {
            assert!(data.is_array());
            let vec_more_attributes = data.as_array().unwrap();
            assert_eq!(vec_more_attributes[0], "and");
            assert_eq!(vec_more_attributes[1], "more");
            assert_eq!(vec_more_attributes[2], "and");
            assert_eq!(vec_more_attributes[3], "more");
        },
        None => panic!("No delivery/config/unlimited, {}", dna_data)
    };
});
