use std::error;

pub enum ErrorKind {
    NoMatchingCommand,
    NotOnABranch,
    CannotReviewSameBranch,
    FailedToExecute,
    PushFailed,
    BadGitOutputMatch,
    NoConfig,
    GitFailed,
    GitSetupFailed,
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
            GitFailed => "Git command failed!",
            GitSetupFailed => "Setup failed; you have already set up delivery.",
            BadGitOutputMatch => "A line of git porcelain did not match!",
            NoConfig => "Cannot find a .git/config file",
        }
    }

    fn detail(&self) -> Option<String> {
        self.detail.clone()
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause()
    }
}

