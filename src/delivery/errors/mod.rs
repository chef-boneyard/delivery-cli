#![allow(unstable)]
use std::error;
use std::io;

#[derive(Show)]
pub enum Kind {
    NoMatchingCommand,
    NotOnABranch,
    CannotReviewSameBranch,
    FailedToExecute,
    PushFailed,
    BadGitOutputMatch,
    NoConfig,
    GitFailed,
    GitSetupFailed,
    ConfigParse,
    MissingConfig,
    ConfigValidation,
    IoError(io::IoError)
}

#[derive(Show)]
pub struct DeliveryError {
    pub kind: Kind,
    pub detail: Option<String>,
}

impl error::Error for DeliveryError {
    fn description(&self) -> &str {
        match self.kind {
            Kind::NoMatchingCommand => "No command matches your arguments - likely unimplemented feature",
            Kind::NotOnABranch => "You must be on a branch",
            Kind::CannotReviewSameBranch => "You cannot target code for review from the same branch as the review is targeted for",
            Kind::FailedToExecute => "Tried to fork a process, and failed",
            Kind::PushFailed => "Git Push failed!",
            Kind::GitFailed => "Git command failed!",
            Kind::GitSetupFailed => "Setup failed; you have already set up delivery.",
            Kind::BadGitOutputMatch => "A line of git porcelain did not match!",
            Kind::NoConfig => "Cannot find a .git/config file",
            Kind::ConfigParse => "Failed to parse the cli config file",
            Kind::MissingConfig => "A configuration value is missing",
            Kind::ConfigValidation => "A required option is missing - use the command line options or 'delivery setup'",
            Kind::IoError(_) => "An I/O Error occured"
        }
    }

    fn detail(&self) -> Option<String> {
        self.detail.clone()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause()
    }
}

impl error::FromError<io::IoError> for DeliveryError {
    fn from_error(err: io::IoError) -> DeliveryError {
        DeliveryError{
            kind: Kind::IoError(err),
            detail: None
        }
    }
}
