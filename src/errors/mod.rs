extern crate git2;

use std::error;
use std::error::FromError;

pub enum ErrorKind {
    NoMatchingCommand,
    NotOnABranch,
    CannotReviewSameBranch,
    FailedToExecute,
    PushFailed,
    BadGitOutputMatch,
    GitError(git2::Error),
}

pub struct DeliveryError {
    pub kind: ErrorKind,
    pub detail: Option<String>,
}

impl error::Error for DeliveryError {
    fn description(&self) -> &str {
        match self.kind {
            NoMatchingCommand => "No command matches your arguments - likely unimplemented feature",
            NotOnABranch => "You must be on a branch",
            CannotReviewSameBranch => "You cannot target code for review from the same branch as the review is targeted for",
            FailedToExecute => "Tried to fork a process, and failed",
            PushFailed => "Git Push failed!",
            BadGitOutputMatch => "A line of git porcelain did not match!",
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

