//
// Copyright:: Copyright (c) 2015 Chef Software, Inc.
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

pub use errors;

use std::process::Command;
use utils::say::{say, sayln, Spinner};
use errors::{DeliveryError, Kind};
use std::env;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::error;
use std::fs::PathExt;

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
// Where P: AsRef<Path> == Any type that implements the AsRef<Path> trait
pub fn git_command<P: ?Sized>(args: &[&str], c: &P) -> Result<GitResult, DeliveryError> where P: AsRef<Path> {
    let cwd = c.as_ref();
    let spinner = Spinner::start();
    let mut command = Command::new("git");
    command.args(args);
    command.current_dir(cwd);
    debug!("Git command: {:?}", command);
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => { spinner.stop(); return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute git: {}", error::Error::description(&e)))}) },
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

#[derive(Debug, Clone)]
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
    pub url: Option<String>,
    pub change_id: Option<String>
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
    let mut r_change_id = None;
    for line in push_error.lines_any() {
        debug!("error: {}", line);
        if line.starts_with("remote") {
            // this weird regex accounts for the fact that some versions of git
            // (at least 1.8.5.2 (Apple Git-48), but possibly others) append the
            // ANSI code ESC[K to every line of the remote's answer when pushing
            let r = regex!(r"remote: ([^ \x{1b}]+)(?:\x{1b}\[K)?$");
            let caps_result = r.captures(line);
            match caps_result {
                Some(caps) => {
                    let cap = caps.at(1).unwrap();
                    if cap.starts_with("http") {
                        let change_url = cap.trim().to_string();
                        r_url = Some(change_url.clone());
                        let change_id_regex = regex!(r"/([a-f0-9]{8}-(?:[a-f0-9]{4}-){3}[a-f0-9]{12})");
                        let change_id_match = change_id_regex.captures(change_url.as_str());
                        r_change_id = Some(String::from_str(change_id_match.unwrap().at(1).unwrap()));
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
                      url: r_url,
                      change_id: r_change_id })
}

pub fn delivery_ssh_url(user: &str, server: &str, ent: &str, org: &str, proj: &str) -> String {
    format!("ssh://{}@{}@{}:8989/{}/{}/{}", user, ent, server, ent, org, proj)
}

pub fn init_repo(path: &PathBuf) -> Result<(), DeliveryError> {
    say("white", "Is ");
    say("magenta", &format!("{} ", path.display()));
    say("white", "a git repo?  ");

    let git_dir = Path::new("./.git");

    if git_dir.exists() {
        sayln("white", "yes");
        return Ok(())
    } else {
        sayln("red", "no. Run 'git init' here and then 'delivery init' again.");
        return Err(DeliveryError{ kind: Kind::GitSetupFailed, detail: None })
    }
}

// This function is not currently used, but will be when we
// add a --force option to the init command.
pub fn create_repo(path: &PathBuf) -> Result<(), DeliveryError> {
    say("white", "Creating repo in: ");
    say("magenta", &format!("{} ", path.display()));
    let result = git_command(&["init"], path);
    match result {
        Ok(_) => {
            sayln("white", "'git init' done.");
            return Ok(());
        },
        Err(e) => return Err(e)
    }
}

pub fn config_repo(user: &str, server: &str, ent: &str, org: &str, proj: &str, path: &PathBuf) -> Result<bool, DeliveryError> {
    sayln("white", &format!("adding remote: {}", &delivery_ssh_url(user, server, ent, org, proj)));
    let url = delivery_ssh_url(user, server, ent, org, proj);
    let result = git_command(&["remote", "add", "delivery", &url], path);
    match result {
        Ok(_) => return Ok(true),
        Err(e) => {
            match e.detail.clone() {
                Some(msg) => {
                    if msg.contains("remote delivery already exists") {
                        return Ok(false);
                    } else {
                        return Err(e)
                    }
                },
                None => {
                    return Err(e)
                }
            }
        },
    }
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

pub fn server_content() -> bool {
    match git_command(&["ls-remote", "delivery", "refs/heads/master"], &cwd()) {
        Ok(msg) => {
            if msg.stdout.contains("refs/heads/master") {
                say("red", &format!("{}", msg.stdout));
                return true
            } else {
                sayln("white", "No upstream content");
                return false
            }
        },
        Err(e) => {
            sayln("red", &format!("got error {:?}", e));
            return false
        }
    }
}

pub fn git_push_master() -> Result<(), DeliveryError> {
    match git_command(&["push", "--set-upstream",
                       "--porcelain", "--progress",
                       "--verbose", "delivery", "master"],
                      &cwd()) {
        Ok(msg) => {
            sayln("white", &format!("{}", msg.stdout));
            return Ok(())
        },
        Err(e) => {
            match e.detail {
                Some(msg) => {
                    if msg.contains("failed to push some refs") {
                        sayln("red", &format!("Failed to push; perhaps there are no local commits?"));
                    }
                    return Ok(())
                },
                None => {
                    return Err(e)
                }
            }
        }
    }
}
