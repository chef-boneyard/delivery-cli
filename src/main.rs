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

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    debug!("{}", args);
    let cmd_result = match args {
        Args {
            cmd_review: true, flag_for: ref for_pipeline, ..
        } => review(for_pipeline),
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

fn review(for_pipeline: &String) -> Result<bool, DeliveryError> {
    debug!("Starting review for pipeline {}", for_pipeline);
    let repo = try!(get_repository());
    let head = try!(get_head(repo));
    println!("Head is {}", head);
    say_greenln(format!("Delivery Review, current branch {}", head.as_slice()).as_slice());
    // say_greenln(head);
    Ok(true)
}

