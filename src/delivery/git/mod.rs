pub use errors;

use std::process::Command;
use utils::say::{say, sayln, Spinner};
use errors::{DeliveryError, Kind};
use std::env;
use std::path::{AsPath, PathBuf};

fn cwd() -> PathBuf {
    env::current_dir().unwrap()
}

pub fn get_head() -> Result<String, DeliveryError> {
    let gitr = try!(git_command(&["branch"], &cwd()));
    let result = try!(parse_get_head(&gitr.stdout));
    Ok(result)
}

fn parse_get_head(stdout: &str) -> Result<String, DeliveryError> {
    for line in stdout.lines_any() {
        let r = regex!(r"(.) (.+)");
        let caps_result = r.captures(line);
        let caps = match caps_result {
            Some(caps) => caps,
            None => { return Err(DeliveryError{ kind: Kind::BadGitOutputMatch, detail: Some(format!("Failed to match: {}", line)) }) }
        };
        let token = caps.at(1).unwrap();
        if token == "*" {
            let branch = caps.at(2).unwrap();
            return Ok(String::from_str(branch));
        }
    }
    return Err(DeliveryError{ kind: Kind::NotOnABranch, detail: None });
}

#[test]
fn test_parse_get_head() {
    let stdout = "  adam/review
  adam/test
  adam/test6
  builder
  first
  foo
  foo2
* master
  snazzy
  testerton";
    let result = parse_get_head(stdout);
    match result {
        Ok(branch) => {
            assert_eq!(&branch[..], "master");
        },
        Err(_) => panic!("No result")
    };
}

pub struct GitResult {
    pub stdout: String,
    pub stderr: String
}

// What is this crazy type signature, you ask? Let me explain!
//
// Where <P: ?Sized> == Any Type (Sized or Unsized)
// Where P: AsPath == Any type that implements the AsPath trait
pub fn git_command<P: ?Sized>(args: &[&str], c: &P) -> Result<GitResult, DeliveryError> where P: AsPath {
    let cwd = c.as_path();
    let spinner = Spinner::start();
    let mut command = Command::new("git");
    command.args(args);
    command.current_dir(cwd);
    debug!("Git command: {:?}", command);
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => { spinner.stop(); return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute git: {}", e.description()))}) },
    };
    debug!("Git exited: {}", output.status);
    spinner.stop();
    if !output.status.success() {
        return Err(DeliveryError{ kind: Kind::GitFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))});
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    debug!("Git stdout: {}", stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    debug!("Git stderr: {}", stderr);
    Ok(GitResult{ stdout: stdout, stderr: stderr })
}

pub fn git_push_review(branch: &str,
                       target: &str) -> Result<ReviewResult, DeliveryError> {
    let gitr = try!(git_command(&["push",
                                  "--porcelain", "--progress",
                                  "--verbose", "delivery",
                                  &format!("{}:_for/{}/{}",
                                           branch, target, branch)],
                                &cwd()));
    parse_git_push_output(&gitr.stdout, &gitr.stderr)
}

/// Output via `sayln` results of a git push.
pub fn say_push_results(results: Vec<PushResult>) {
    for result in results.iter() {
        match result.flag {
            PushResultFlag::SuccessfulFastForward =>
                sayln("green", &format!("Updated change: {}", result.reason)),
            PushResultFlag::SuccessfulForcedUpdate =>
                sayln("green",
                      &format!("Force updated change: {}", result.reason)),
            PushResultFlag::SuccessfulDeletedRef =>
                sayln("red", &format!("Deleted change: {}", result.reason)),
            PushResultFlag::SuccessfulPushedNewRef =>
                sayln("green", &format!("Created change: {}", result.reason)),
            PushResultFlag::Rejected =>
                sayln("red", &format!("Rejected change: {}", result.reason)),
            PushResultFlag::UpToDate =>
                sayln("yellow",
                      &format!("Nothing added to the existing change")),
        }
    }
}

#[derive(Debug)]
pub enum PushResultFlag {
    SuccessfulFastForward,
    SuccessfulForcedUpdate,
    SuccessfulDeletedRef,
    SuccessfulPushedNewRef,
    Rejected,
    UpToDate,
}

impl Copy for PushResultFlag { }

/// Returned by `git_push_review`. The `push_results` field is a
/// vector of `PushResult` each indicating a `PushResultFalg` and a
/// reason message. The `messages` field is a vector of output lines
/// returned from the server managing the git protocol (as you'd see
/// on the command line prefixed with `remote: $LINE`. The `url` field
/// will contain the last line that looks like a URL returned as
/// remote data.
#[derive(Debug)]
pub struct ReviewResult {
    pub push_results: Vec<PushResult>,
    pub messages: Vec<String>,
    pub url: Option<String>
}

#[derive(Debug)]
pub struct PushResult {
    flag: PushResultFlag,
    reason: String
}

pub fn parse_git_push_output(push_output: &str,
                             push_error: &str) -> Result<ReviewResult, DeliveryError> {
    let mut push_results: Vec<PushResult> = Vec::new();
    let mut r_messages: Vec<String> = Vec::new();
    let mut r_url = None;
    for line in push_error.lines_any() {
        debug!("error: {}", line);
        if line.starts_with("remote") {
            let r = regex!(r"remote: ([^ ]+)");
            let caps_result = r.captures(line);
            match caps_result {
                Some(caps) => {
                    let cap = caps.at(1).unwrap();
                    if cap.starts_with("http") {
                        r_url = Some(cap.trim().to_string());
                    } else {
                        r_messages.push(cap.to_string());
                    }
                 },
                None => {}
            }
        }
    }
    for line in push_output.lines_any() {
        debug!("output: {}", line);
        if line.starts_with("To") {
            continue;
        } else if line.starts_with("Done") {
            continue;
        }
        let r = regex!(r"(.)\t(.+):(.+)\t\[(.+)\]");
        let caps_result = r.captures(line);
        let caps = match caps_result {
            Some(caps) => caps,
            None => {
                let detail = Some(format!("Failed to match: {}", line));
                return Err(DeliveryError{ kind: Kind::BadGitOutputMatch,
                                          detail: detail })
            }
        };
        let result_flag = match caps.at(1).unwrap() {
            " " => PushResultFlag::SuccessfulFastForward,
            "+" => PushResultFlag::SuccessfulForcedUpdate,
            "-" => PushResultFlag::SuccessfulDeletedRef,
            "*" => PushResultFlag::SuccessfulPushedNewRef,
            "!" => PushResultFlag::Rejected,
            "=" => PushResultFlag::UpToDate,
            _ => {
                return Err(DeliveryError{
                    kind: Kind::BadGitOutputMatch,
                    detail: Some(format!("Unknown result flag"))})
            }
        };
        push_results.push(
            PushResult{
                flag: result_flag,
                reason: String::from_str(caps.at(4).unwrap())
            }
            )
    }
    Ok(ReviewResult { push_results: push_results,
                      messages: r_messages,
                      url: r_url })
}

pub fn delivery_ssh_url(user: &str, server: &str, ent: &str, org: &str, proj: &str) -> String {
    format!("ssh://{}@{}@{}:8989/{}/{}/{}", user, ent, server, ent, org, proj)
}

pub fn config_repo(user: &str, server: &str, ent: &str, org: &str, proj: &str, path: &PathBuf) -> Result<(), DeliveryError> {
    let result = git_command(&["remote", "add", "delivery", &delivery_ssh_url(user, server, ent, org, proj)], path);
    match result {
        Ok(_) => return Ok(()),
        Err(e) => {
            match e.detail {
                Some(msg) => {
                    if msg.contains("remote delivery already exists") {
                        return Err(DeliveryError{ kind: Kind::GitSetupFailed, detail: None });
                    }
                },
                None => {
                    return Err(e)
                }
            }
        },
    }
    Ok(())
}

pub fn checkout_branch_name(change: &str, patchset: &str) -> String {
    if patchset == "latest" {
        return String::from_str(change);
    } else {
        return format!("{}/{}", change, patchset);
    }
}

pub fn diff(change: &str, patchset: &str, pipeline: &str, local: &bool) -> Result<(), DeliveryError> {
    try!(git_command(&["fetch", "delivery"], &cwd()));
    let mut first_branch = format!("delivery/{}", pipeline);
    if *local {
        first_branch = String::from_str("HEAD");
    }
    let diff = try!(git_command(&["diff", "--color=always", &first_branch, &format!("delivery/_reviews/{}/{}/{}", pipeline, change, patchset)], &cwd()));
    say("white", "\n");
    sayln("white", &diff.stdout);
    Ok(())
}

pub fn clone(project: &str, git_url: &str) -> Result<(), DeliveryError> {
    try!(git_command(&["clone", git_url, project], &cwd()));
    Ok(())
}

pub fn checkout_review(change: &str, patchset: &str, pipeline: &str) -> Result<(), DeliveryError> {
    try!(git_command(&["fetch", "delivery"], &cwd()));
    let branchname = checkout_branch_name(change, patchset);
    let result = git_command(&["branch", "--track", &branchname, &format!("delivery/_reviews/{}/{}/{}", pipeline, change, patchset)], &cwd());
    match result {
        Ok(_) => {
            try!(git_command(&["checkout", &branchname], &cwd()));
            return Ok(())
        },
        Err(e) => {
            match e.detail {
                Some(msg) => {
                    if msg.contains("already exists.") {
                        try!(git_command(&["checkout", &branchname], &cwd()));
                        sayln("white", "Branch already exists, checking it out.");
                        let r = try!(git_command(&["status"], &cwd()));
                        sayln("white", &r.stdout);
                        return Ok(())
                    } else {
                        return Err(DeliveryError{kind: Kind::GitFailed, detail: Some(msg)});
                    }
                },
                None => {
                    return Err(e)
                }
            }
        },
    }
}
