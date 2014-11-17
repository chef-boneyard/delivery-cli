#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate git2;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin, link)] extern crate log;
extern crate term;

pub use errors;

use git2::Repository;
use std::os;
use std::io::process::Command;
use utils::say::sayln;
use errors::{DeliveryError};

pub fn get_repository() -> Result<git2::Repository, DeliveryError> {
    let repo = try!(git2::Repository::discover(&os::getcwd()));
    Ok(repo)
}

pub fn get_head(repo: git2::Repository) -> Result<String, DeliveryError> {
    let head = try!(repo.head());
    let shorthand = head.shorthand();
    let result = match shorthand {
        Some(result) => Ok(String::from_str(result)),
        None => Err(DeliveryError{ kind: errors::NotOnABranch, detail: None })
    };
    result
}

pub fn git_push(branch: &str, target: &str) -> Result<String, DeliveryError> {
    let mut command = Command::new("git");
    command.arg("push");
    command.arg("--porcelain");
    command.arg("origin");
    command.arg(format!("{}:_for/{}/{}", branch, target, branch));
    debug!("Running: {}", command);
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => { return Err(DeliveryError{ kind: errors::FailedToExecute, detail: Some(format!("failed to execute git: {}", e.desc))}) },
    };
    if !output.status.success() {
        return Err(DeliveryError{ kind: errors::PushFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(output.output.as_slice()), String::from_utf8_lossy(output.error.as_slice())))});
    }
    let stdout = String::from_utf8_lossy(output.output.as_slice()).into_string();
    debug!("Git push: {}", stdout);
    debug!("Git exited: {}", output.status);
    let output = try!(parse_git_push_output(stdout.as_slice()));
    for result in output.iter() {
        match result.flag {
            SuccessfulFastForward => sayln("green", format!("Updated change: {}", result.reason).as_slice()),
            SuccessfulForcedUpdate => sayln("green", format!("Force updated change: {}", result.reason).as_slice()),
            SuccessfulDeletedRef => sayln("red", format!("Deleted change: {}", result.reason).as_slice()),
            SuccessfulPushedNewRef => sayln("green", format!("Created change: {}", result.reason).as_slice()),
            Rejected => sayln("red", format!("Rejected change: {}", result.reason).as_slice()),
            UpToDate => sayln("yellow", format!("Nothing added to the existing change").as_slice()),
        }
    }
    Ok(stdout.into_string())
}

pub enum PushResultFlags {
    SuccessfulFastForward,
    SuccessfulForcedUpdate,
    SuccessfulDeletedRef,
    SuccessfulPushedNewRef,
    Rejected,
    UpToDate,
}

pub struct PushResult {
    flag: PushResultFlags,
    from: String,
    to: String,
    reason: String
}

pub fn parse_git_push_output(push_output: &str) -> Result<Vec<PushResult>, DeliveryError> {
    let mut push_results: Vec<PushResult> = Vec::new();
    for line in push_output.lines_any() {
        debug!("{}", line);
        if line.starts_with("To") {
            continue;
        } else if line.starts_with("Done") {
            continue;
        }
        let r = regex!(r"(.)\t(.+):(.+)\t\[(.+)\]");
        let caps_result = r.captures(line);
        let caps = match caps_result {
            Some(caps) => caps,
            None => { return Err(DeliveryError{ kind: errors::BadGitOutputMatch, detail: Some(format!("Failed to match: {}", line)) }) }
        };
        let result_flag = match caps.at(1) {
            " " => SuccessfulFastForward,
            "+" => SuccessfulForcedUpdate,
            "-" => SuccessfulDeletedRef,
            "*" => SuccessfulPushedNewRef,
            "!" => Rejected,
            "=" => UpToDate,
            _ => { return Err(DeliveryError{ kind: errors::BadGitOutputMatch, detail: Some(format!("Unknown result flag")) }) }
        };
        push_results.push(
            PushResult{
                flag: result_flag,
                from: String::from_str(caps.at(2)),
                to: String::from_str(caps.at(3)),
                reason: String::from_str(caps.at(4))
            }
        )
    }
    Ok(push_results)
}

#[test]
fn test_parse_git_push_output() {
    let input = "To ssh://adam@127.0.0.1/Users/adam/src/opscode/delivery/opscode/delivery-cli2
=	refs/heads/foo:refs/heads/_for/master/foo	[up to date]
Done";
    let results: Vec<String> = parse_git_push_output(input);
    let mut valid_result: Vec<String> = Vec::new();
    valid_result.push(String::from_str("Review branch _for/master/foo is up to date"));
    assert_eq!(valid_result, results);
}

