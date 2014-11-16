#![feature(phase)]

extern crate serialize;
extern crate git2;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin, link)] extern crate log;
extern crate term;

use git2::Repository;
use std::os;
use std::error;
use std::error::FromError;

docopt!(Args deriving Show, "
Usage: delivery review [--for=<pipeline>]
       delivery checkout <change> [--patchset=<number>]
       delivery rebase [--for=<pipeline>]
       delivery --help

Options:
  -h, --help               Show this message.
  -f, --for=<pipeline>     A pipeline to target [default: master]
  -p, --patchset=<number>  A patchset number [default: latest]
")

#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    debug!("{}", args);
    let cmd_result = match args {
        Args {
            cmd_review: true, flag_for: ref for_pipeline, ..
        } => review(for_pipeline.as_slice()),
        _ => no_matching_command(),
    };
    match cmd_result {
        Ok(_) => {},
        Err(e) => exit_with(e, 1)
    }
}

enum ErrorKind {
    NoMatchingCommand,
    NotOnABranch,
    GitError(git2::Error),
}

struct DeliveryError {
    pub kind: ErrorKind,
    pub detail: Option<String>,
}

impl error::Error for DeliveryError {
    fn description(&self) -> &str {
        match self.kind {
            NoMatchingCommand => "No command matches your arguments - likely unimplemented feature",
            NotOnABranch => "You must be on a branch",
            GitError(_) => "A git error occured",
        }
    }

    fn detail(&self) -> Option<String> {
        self.detail.clone()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause()
    }
}

impl FromError<git2::Error> for DeliveryError {
    fn from_error(err: git2::Error) -> DeliveryError {
        let message = err.message().clone();
        DeliveryError {
            kind: GitError(err),
            detail: Some(message),
        }
    }
}

fn no_matching_command() -> Result<bool, DeliveryError> {
    Err(DeliveryError { kind: NoMatchingCommand, detail: None })
}

fn say_greenln(to_say: &str) {
    say_green(to_say);
    let mut t = term::stdout().unwrap();
    (write!(t, "\n")).unwrap();
}

fn say_green(to_say: &str) {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::GREEN).unwrap();
    (write!(t, "{}", to_say)).unwrap();
    t.reset().unwrap()
}

fn say_redln(to_say: &str) {
    say_red(to_say);
    let mut t = term::stdout().unwrap();
    (write!(t, "\n")).unwrap();
}

fn say_red(to_say: &str) {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::RED).unwrap();
    (write!(t, "{}", to_say)).unwrap();
    t.reset().unwrap()
}

fn get_repository() -> Result<Repository, DeliveryError> {
    let repo = try!(Repository::discover(&os::getcwd()));
    Ok(repo)
}

fn get_head(repo: Repository) -> Result<String, DeliveryError> {
    let head = try!(repo.head());
    let shorthand = head.shorthand();
    let result = match shorthand {
        Some(result) => Ok(String::from_str(result)),
        None => Err(DeliveryError{ kind: NotOnABranch, detail: None })
    };
    result
}

fn exit_with<T: error::Error>(e: T, i: int) {
    say_redln(e.description());
    match e.detail() {
        Some(deets) => say_redln(deets.as_slice()),
        None => {}
    }
    os::set_exit_status(i)
}

fn git_push(branch: &str, target: &str) -> Result<String, DeliveryError> {
    let mut command = Command::new("git");
    command.arg("push");
    command.arg("--porcelain");
    command.arg("origin");
    command.arg(format!("{}:_for/{}/{}", branch, target, branch));
    debug!("Running: {}", command);
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => { return Err(DeliveryError{ kind: FailedToExecute, detail: Some(format!("failed to execute git: {}", e.desc))}) },
    };
    if !output.status.success() {
        return Err(DeliveryError{ kind: PushFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(output.output.as_slice()), String::from_utf8_lossy(output.error.as_slice())))});
    }
    let stdout = String::from_utf8_lossy(output.output.as_slice()).into_string();
    parse_git_push_output(stdout.as_slice());
    debug!("Git push: {}", stdout);
    debug!("Git exited: {}", output.status);
    Ok(stdout.into_string())
}

fn parse_git_push_output(push_output: &str) {
    for line in push_output.lines_any() {
        println!("piece: {}", line)
    }
}

#[test]
fn test_parse_git_push_output() {
    let input = String::from_str("To ssh://adam@127.0.0.1/Users/adam/src/opscode/delivery/opscode/delivery-cli2
=	refs/heads/foo:refs/heads/_for/master/foo	[up to date]
Done");
    let results: Vec<String> = parse_git_push_output(input);
    let mut valid_result: Vec<String> = Vec::new();
    valid_result.push(String::from_str("Review branch _for/master/foo is up to date"));
    assert_eq!(valid_result, results);
}

fn review(for_pipeline: &str) -> Result<bool, DeliveryError> {
    let repo = try!(get_repository());
    let head = try!(get_head(repo));
    if for_pipeline == head.as_slice() {
        return Err(DeliveryError{ kind: CannotReviewSameBranch, detail: None })
    }
    say("green", "Delivery");
    say("white", " review for change ");
    say("yellow", head.as_slice());
    say("white", " targeted for pipeline ");
    sayln("magenta", for_pipeline.as_slice());
    let output = try!(git_push(head.as_slice(), for_pipeline));
    sayln("white", output.as_slice());
    Ok(true)
}

