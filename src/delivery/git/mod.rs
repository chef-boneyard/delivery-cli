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
use utils::path_ext::{is_dir};
use errors::{DeliveryError, Kind};
use std::env;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::error;
use regex::Regex;

fn cwd() -> PathBuf {
    env::current_dir().unwrap()
}

pub fn get_head() -> Result<String, DeliveryError> {
    let gitr = try!(git_command(&["branch"], &cwd()));
    let result = try!(parse_get_head(&gitr.stdout));
    Ok(result)
}

fn parse_get_head(stdout: &str) -> Result<String, DeliveryError> {
    for line in stdout.lines() {
        let r = Regex::new(r"(.) (.+)").unwrap();
        let caps_result = r.captures(line);
        let caps = match caps_result {
            Some(caps) => caps,
            None => { return Err(DeliveryError{ kind: Kind::BadGitOutputMatch, detail: Some(format!("Failed to match: {}", line)) }) }
        };
        let token = caps.at(1).unwrap();
        if token == "*" {
            let branch = caps.at(2).unwrap();
            return Ok(String::from(branch));
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

impl Default for ReviewResult {
    fn default() -> ReviewResult {
        ReviewResult { push_results: Vec::new(),
                       messages: Vec::new(),
                       url: None,
                       change_id: None }
    }
}

#[derive(Debug)]
pub struct PushResult {
    flag: PushResultFlag,
    reason: String
}

pub fn parse_git_push_output(push_output: &str,
                             push_error: &str) -> Result<ReviewResult, DeliveryError> {
    let mut review_result = ReviewResult::default();
    for line in push_error.lines() {
        debug!("error: {}", line);
        if line.starts_with("remote") {
            parse_line_from_remote(&line, &mut review_result);
        }
    }
    for line in push_output.lines() {
        debug!("output: {}", line);
        if line.starts_with("To") ||  line.starts_with("Done") {
            continue;
        }
        let r = Regex::new(r"(.)\t(.+):(.+)\t\[(.+)\]").unwrap();
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
        review_result.push_results.push(
            PushResult{
                flag: result_flag,
                reason: String::from(caps.at(4).unwrap())
            })
    }
    Ok(review_result)
}

/// Parses a line returned by the remote
fn parse_line_from_remote(line: &str, review_result: &mut ReviewResult) -> () {
    // this weird regex accounts for the fact that some versions of git
    // (at least 1.8.5.2 (Apple Git-48), but possibly others) append the
    // ANSI code ESC[K to every line of the remote's answer when pushing
    let r = Regex::new(r"remote: ([^\x{1b}]+)(?:\x{1b}\[K)?$").unwrap();
    let caps_result = r.captures(line);
    match caps_result {
        Some(caps) => {
            let cap = caps.at(1).unwrap();
            if cap.starts_with("http") {
                let change_url = cap.trim().to_string();
                review_result.url = Some(change_url.clone());
                let change_id_regex = Regex::new(r"/([a-f0-9]{8}-(?:[a-f0-9]{4}-){3}[a-f0-9]{12})").unwrap();
                let change_id_match = change_id_regex.captures(&change_url);
                review_result.change_id = Some(String::from(change_id_match.unwrap().at(1).unwrap()));
            } else {
                review_result.messages.push(cap.to_string());
            }
         },
        None => {}
    }
}

pub fn check_repo_init(path: &PathBuf) -> Result<(), DeliveryError> {
    say("white", "Is ");
    say("magenta", &format!("{} ", path.display()));
    say("white", "a git repo?  ");

    let git_dir = path.join(".git");

    if is_dir(git_dir.as_path()) {
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

pub fn config_repo(url: &str, path: &PathBuf) -> Result<bool, DeliveryError> {
    let result = git_command(&["remote", "add", "delivery", &url], path);
    match result {
        Ok(_) => return Ok(true),
        Err(e) => {
            match e.detail.clone() {
                Some(msg) => {
                    if msg.contains("remote delivery already exists") {
                        // Check to see if the current delivery git remote matches
                        // the url passed in.
                        let git_version_result = git_command(&["remote", "-v", "show", "-n", "delivery"], path);
                        match git_version_result {
                            Ok(git_result) => {
                                if git_result.stdout.contains(url) {
                                    return Ok(false);
                                } else {
                                    return Err(DeliveryError {
                                        kind: Kind::GitFailed,
                                        detail: Some(remote_already_exists_error_msg(url))
                                    });
                                }
                            },
                            Err(e) => return Err(e)
                        }
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

fn remote_already_exists_error_msg(url: &str) -> String {
    let error = "A git remote named 'delivery' already exists in this repo, but it is different than what was contained in your config file:\n\n".to_string() + url;
    return error + "\n\nPlease either update your cli.toml or your git remote. Run:\n\ngit remote -v show -n delivery\n\nto see your current delivery remote."
}

pub fn checkout_branch_name(change: &str, patchset: &str) -> String {
    if patchset == "latest" {
        return String::from(change);
    } else {
        return format!("{}/{}", change, patchset);
    }
}

pub fn diff(change: &str, patchset: &str, pipeline: &str, local: &bool) -> Result<(), DeliveryError> {
    try!(git_command(&["fetch", "delivery"], &cwd()));
    let mut first_branch = format!("delivery/{}", pipeline);
    if *local {
        first_branch = String::from("HEAD");
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

// Verify the content of the repo:pipeline on the server
pub fn server_content(pipeline: &str) -> Result<bool, DeliveryError> {
    let p_ref = &format!("refs/heads/{}", pipeline);
    match git_command(&["ls-remote", "delivery", p_ref], &cwd()) {
        Ok(msg) => {
            if msg.stdout.contains(p_ref) {
                return Ok(true)
            } else {
                return Ok(false)
            }
        },
        Err(e) => return Err(e)
    }
}

// Push pipeline content to the Server
pub fn git_push(pipeline: &str) -> Result<(), DeliveryError> {
    // Check if the pipeline branch exists and has commits.
    // If the pipeline branch exists and does not have commits,
    // then `git branch` will not return it, so just checking
    // `git branch` output will handle both cases (pipeline does
    // not exist and pipeline exists but without commits).
    match git_command(&["branch"], &cwd()) {
        Ok(msg) => {
            if !msg.stdout.contains(pipeline) {
                sayln("red", &format!("A {} branch does not exist locally.", pipeline));
                sayln("red", &format!("A {} branch with commits is needed to create the {} \
                                      pipeline.\n", pipeline, pipeline));
                sayln("red", &format!("If your project already has git history, you should \
                                      pull it into {} locally.", pipeline));
                sayln("red", &format!("For example, if your remote is named origin, and your \
                                      git history is in {} run:\n", pipeline));
                sayln("red", &format!("git pull origin {}\n", pipeline));
                sayln("red", "However, if this is a brand new project, make an initial commit by running:\n");
                sayln("red", &format!("git checkout -b {}", pipeline));
                sayln("red", "git commit --allow-empty -m 'Initial commit.'\n");
                sayln("red", &format!("Once you have commits on the {} branch, run `delivery \
                                      init` again.", pipeline));
                return Err(DeliveryError{ kind: Kind::GitFailed, detail: None });
            }
            true
        },
        Err(e) => return Err(e)
    };

    // Master branch exists with commits on it, push it up so the master pipeline can be made.
    match git_command(&["push", "--set-upstream",
                        "--porcelain", "--progress",
                        "--verbose", "delivery", pipeline],
                      &cwd()) {
        Ok(_) => return Ok(()),
        // Not expecting any errors at this point.
        Err(e) => return Err(e)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReviewResult, parse_line_from_remote, check_repo_init};
    use std::path::PathBuf;
    use std::fs::DirBuilder;

    #[test]
    fn test_check_repo_init_with_invalid_path() {
        let path = PathBuf::from("/tmp/not_real");
        assert!(check_repo_init(&path).is_err());
    }

    #[test]
    fn test_check_repo_init_with_valid_path_no_git() {
        let path = PathBuf::from("/tmp/real1");
        DirBuilder::new()
            .recursive(true)
            .create(&path).unwrap();
        assert!(check_repo_init(&path).is_err());
    }

    #[test]
    fn test_check_repo_init_with_valid_path() {
        let path = PathBuf::from("/tmp/real2/");
        let full_path = path.join(".git");
        DirBuilder::new()
            .recursive(true)
            .create(&full_path).unwrap();
        assert!(check_repo_init(&path).is_ok());
    }

    #[test]
    fn test_parse_line_from_remote() {
        test_parse_line_from_remote_with_eol("");
        // older git versions add this ANSI escape code at the end of the lines
        test_parse_line_from_remote_with_eol("\u{1b}[K");
    }

    fn test_parse_line_from_remote_with_eol(remote_msg_eol: &str) {
        let mut review_result = ReviewResult::default();

        // a random message line
        let random_msg = "A random message";
        let line1 = format!("remote: {}{}", random_msg, remote_msg_eol);
        parse_line_from_remote(&line1, &mut review_result);
        assert_eq!(random_msg, review_result.messages[0]);

        // a change URL line
        let change_id = "4bc3f44f-d81f-48a5-bd38-2c7963cb6d94";
        let change_url = format!("https://delivery.shd.chef.co/e/Chef/#/organizations/sandbox/projects/radar/changes/{}", change_id);
        let line2 = format!("remote: {}{}", change_url, remote_msg_eol);
        parse_line_from_remote(&line2, &mut review_result);
        assert_eq!(change_url, review_result.url.unwrap());
        assert_eq!(change_id, review_result.change_id.unwrap());
    }
}
