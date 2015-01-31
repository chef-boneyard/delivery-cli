#![allow(unstable)]
use rustc_serialize::json;
use std::error::{self, Error};
use std::old_io;
use std::fmt;

#[derive(Debug)]
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
    IoError(old_io::IoError),
    JsonError(json::ParserError),
    JsonEncode(json::EncoderError),
    NoBuildCookbook,
    NoHomedir,
    BerksFailed
}

#[derive(Debug)]
pub struct DeliveryError {
    pub kind: Kind,
    pub detail: Option<String>,
}

impl DeliveryError {
    pub fn detail(&self) -> Option<String> {
        self.detail.clone()
    }
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
            Kind::IoError(_) => "An I/O Error occured",
            Kind::JsonError(_) => "A JSON Parser error occured",
            Kind::JsonEncode(_) => "A JSON Encoding error occured",
            Kind::NoBuildCookbook => "No build_cookbook entry in .delivery/config.json",
            Kind::NoHomedir => "Cannot find a homedir",
            Kind::BerksFailed => "Berkshelf command failed"
        }
    }


    fn cause(&self) -> Option<&error::Error> {
        match self.kind {
            Kind::IoError(ref err) => Some(err as &error::Error),
            Kind::JsonError(ref err) => Some(err as &error::Error),
            Kind::JsonEncode(ref err) => Some(err as &error::Error),
            _ => None
        }
    }
}

impl fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}


impl error::FromError<json::EncoderError> for DeliveryError {
    fn from_error(err: json::EncoderError) -> DeliveryError {
        DeliveryError{
            kind: Kind::JsonEncode(err),
            detail: None
        }
    }
}

impl error::FromError<old_io::IoError> for DeliveryError {
    fn from_error(err: old_io::IoError) -> DeliveryError {
        DeliveryError{
            kind: Kind::IoError(err),
            detail: None
        }
    }
}

impl error::FromError<json::ParserError> for DeliveryError {
    fn from_error(err: json::ParserError) -> DeliveryError {
        DeliveryError{
            kind: Kind::JsonError(err),
            detail: None
        }
    }
}

