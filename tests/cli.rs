use delivery::git::git_command;
use delivery::utils::copy_recursive;
use delivery::utils::say;
use std::io::prelude::*;
use tempdir::TempDir;
use std::fs::{self, File};
use std::path::Path;
use support::paths::fixture_file;
use std::process::{Command, Output};
use std::env;
use rustc_serialize::json::Json;
use delivery::utils::path_join_many::PathJoinMany;

// ** Functions used in tests **

fn setup() {
    say::turn_off_spinner();
}

/// Sets up a mock delivery git project from the test_repo fixture.
/// Includes copying in the `.delivery/config.json` that you plan
/// on using.
fn setup_mock_delivery_project_git(dot_config: &str) -> TempDir {
    let tmpdir = TempDir::new("mock-delivery-remote").unwrap();
    let test_repo_path = fixture_file("test_repo");
    panic_on_error!(copy_recursive(&test_repo_path.join(".delivery"), &tmpdir.path().to_path_buf()));
    panic_on_error!(copy_recursive(&test_repo_path.join("README.md"), &tmpdir.path().to_path_buf()));
    panic_on_error!(copy_recursive(&test_repo_path.join("cookbooks"), &tmpdir.path().to_path_buf()));
    panic_on_error!(copy_recursive(&fixture_file(dot_config), &tmpdir.path().join_many(&[".delivery", "config.json"])));
    panic_on_error!(git_command(&["init", tmpdir.path().to_str().unwrap()], tmpdir.path()));
    panic_on_error!(git_command(&["add", "."], tmpdir.path()));
    panic_on_error!(git_command(&["commit", "-a", "-m", "Initial Commit"], tmpdir.path()));
    tmpdir
}

/// Given a path, it copies the build cookbook into it, and turns it
/// into a git repository
fn setup_build_cookbook_project(tmpdir: &Path) {
    let build_cookbook_path = fixture_file("delivery_test");
    panic_on_error!(copy_recursive(&build_cookbook_path, &tmpdir.to_path_buf()));
    panic_on_error!(git_command(&["init", tmpdir.join("delivery_test").to_str().unwrap()], &tmpdir.join("delivery_test")));
    panic_on_error!(git_command(&["add", "."], &tmpdir.join("delivery_test")));
    panic_on_error!(git_command(&["commit", "-a", "-m", "Initial Commit"], &tmpdir.join("delivery_test")));
}

/// Clones a mock delivery git project to a local copy, as if it was
/// on a workstation. Also creates a mock delivery remote pointing at
/// the on-disk mocked delivery project.
fn setup_local_project_clone(delivery_project_git: &Path) -> TempDir {
    let tmpdir = TempDir::new("local-project").unwrap();
    panic_on_error!(git_command(&["clone",
                                  delivery_project_git.to_str().unwrap(),
                                  tmpdir.path().to_str().unwrap()
                                 ], tmpdir.path()));
    panic_on_error!(git_command(&["remote", "add", "delivery", delivery_project_git.to_str().unwrap()], tmpdir.path()));
    let mut command = delivery_cmd();
    command.arg("setup")
           .arg("--user").arg("cavalera")
           .arg("--server").arg("localhost")
           .arg("--ent").arg("family")
           .arg("--org").arg("sepultura")
           .arg("--for").arg("master")
           .arg("--config-path").arg(tmpdir.path().to_str().unwrap());
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

/// Checks out the named branch
fn setup_checkout_branch(tmpdir: &Path, branch: &str) {
    panic_on_error!(git_command(&["checkout", branch], tmpdir));
}

/// Runs the `Command` in `Dir`, and makes sure it exists with 0
/// Returns the result
fn assert_command_successful(command: &mut Command, dir: &Path) -> Output {
    let result = panic_on_error!(command.current_dir(&dir).output());
    if ! result.status.success() {
        let output = String::from_utf8_lossy(&result.stdout);
        let error = String::from_utf8_lossy(&result.stderr);
        panic!("Failed command {:?}\nOUT: {}\nERR: {}\nPath: {}", command, &output, &error, dir.to_str().unwrap());
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
        panic!("Command {:?} should have failed!\nOUT: {}\nERR: {}\nPath: {}", command, &output, &error, dir.to_str().unwrap());
    };
    result
}

/// Builds the command to run delivery review
fn delivery_review_command(pipeline: &str) -> Command {
    let mut command = delivery_cmd();
    command.arg("review").arg("--no-open").arg("--for").arg(pipeline);
    command
}

/// Builds the command to run a sample job
fn delivery_verify_command(job_root: &Path) -> Command {
    let mut command = delivery_cmd();
    command.arg("job")
           .arg("verify")
           .arg("unit")
           .arg("--no-spinner")
           .arg("--job-root").arg(job_root.to_str().unwrap());
    command
}

/// Calls delivery review, and creates the two stub branches that the
/// api would create (`_reviews/PIPELINE/BRANCH/1` and `_reviews/PIPELINE/BRANCH/latest`)
fn delivery_review(local: &Path, remote: &Path, branch: &str, pipeline: &str) {
    panic_on_error!(git_command(&["checkout", branch], local));
    let mut command = delivery_review_command(pipeline);
    assert_command_successful(&mut command, local);

    // Stub out the behavior of the delivery-api
    panic_on_error!(git_command(&["branch", &format!("_reviews/{}/{}/1", pipeline, branch)], remote));
    panic_on_error!(git_command(&["branch", &format!("_reviews/{}/{}/latest", pipeline, branch)], remote));
}

/// Returns a Command set to the delivery binary created when you
/// ran `cargo test`.
fn delivery_cmd() -> Command {
    let mut delivery_path = env::current_exe().unwrap();
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

// Tests `delivery review`. Fails if the command fails, or if we fail to create
// the remote branch _for/master/rust/test, which is what we need to push to
// the API server.
test!(review {
    let delivery_project_git = setup_mock_delivery_project_git("path_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    delivery_review(&local_project.path(), &delivery_project_git.path(), "rust/test", "master");
    setup_checkout_branch(&delivery_project_git.path(), "_for/master/rust/test");
});

test!(review_from_child_dir {
    let delivery_project_git = setup_mock_delivery_project_git("path_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    let child_dir = local_project.path().join("fuzz-bucket");
    panic_on_error!(fs::create_dir_all(&child_dir));
    delivery_review(&child_dir, &delivery_project_git.path(), "rust/test", "master");
    setup_checkout_branch(&delivery_project_git.path(), "_for/master/rust/test");
});

test!(review_with_a_v1_config {
    let delivery_project_git = setup_mock_delivery_project_git("v1_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    delivery_review(&local_project.path(), &delivery_project_git.path(), "rust/test", "master");
    setup_checkout_branch(&delivery_project_git.path(), "_for/master/rust/test");
});

test!(review_without_dependencies {
    let delivery_project_git = setup_mock_delivery_project_git("no_deps_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    delivery_review(&local_project.path(), &delivery_project_git.path(), "rust/test", "master");
    setup_checkout_branch(&delivery_project_git.path(), "_for/master/rust/test");
});



test!(review_with_an_invalid_config {
    let delivery_project_git = setup_mock_delivery_project_git("invalid_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_review_command("rust/test");
    assert_command_failed(&mut command, &local_project.path());
});

test!(job_verify_unit_with_path_config {
    let delivery_project_git = setup_mock_delivery_project_git("path_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_successful(&mut command, &local_project.path());
});

test!(job_verify_unit_with_git_config {
    let delivery_project_git = setup_mock_delivery_project_git("git_config.json");
    let local_project = setup_local_project_clone(&delivery_project_git.path());
    let job_root = TempDir::new("job-root").unwrap();
    setup_build_cookbook_project(&job_root.path());
    setup_change(&local_project.path(), "rust/test", "freaky");
    let mut command = delivery_verify_command(&job_root.path());
    assert_command_successful(&mut command, &local_project.path());
});

// TODO: This test requires internet access... we should move it out
// into an Acceptance-stage test instead of here in unit tests. It's
// impossible to run on a plane, for instance :(
test!(job_verify_unit_with_public_supermarket_config {
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
test!(job_verify_unit_with_private_supermarket_config {
   let delivery_project_git = setup_mock_delivery_project_git("private_supermarket_config.json");
   let local_project = setup_local_project_clone(&delivery_project_git.path());
   let job_root = TempDir::new("job-root").unwrap();
   setup_change(&local_project.path(), "rust/test", "freaky");
   let mut command = delivery_verify_command(&job_root.path());
   assert_command_successful(&mut command, &local_project.path());
   assert!(job_root.path().join_many(&["chef", "cookbooks", "delivery-truck"]).is_dir());
   assert!(job_root.path().join_many(&["chef", "cookbooks", "delivery-truck", "recipes", "unit.rb"]).is_file());
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
    let dna_data = panic_on_error!(Json::from_str(&dna_json));
    match dna_data.find_path(&["delivery", "workspace", "repo"]) {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_string().unwrap(), job_root.path().join("repo").to_str().unwrap());
        },
        None => panic!("No delivery/workspace/repo, {}", dna_data)
    };
    match dna_data.find_path(&["delivery", "workspace", "chef"]) {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_string().unwrap(), job_root.path().join("chef").to_str().unwrap());
        },
        None => panic!("No delivery/workspace/chef, {}", dna_data)
    };
    match dna_data.find_path(&["delivery", "workspace", "cache"]) {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_string().unwrap(), job_root.path().join("cache").to_str().unwrap());
        },
        None => panic!("No delivery/workspace/cache, {}", dna_data)
    };
    match dna_data.find_path(&["delivery", "workspace", "root"]) {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_string().unwrap(), job_root.path().to_str().unwrap());
        },
        None => panic!("No delivery/workspace/root, {}", dna_data)
    };
    match dna_data.find_path(&["delivery_builder", "build_user"]) {
        Some(data) => {
            assert!(data.is_string());
            assert_eq!(data.as_string().unwrap(), "dbuild");
        },
        None => panic!("No delivery_builderl/build_user, {}", dna_data)
    };
});
