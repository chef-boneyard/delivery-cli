//
// Copyright:: Copyright (c) 2016 Chef Software, Inc.
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

use rustc_serialize::json;
use std::error::{self, Error};
use std::num;
use std::io;
use std::fmt;
use hyper;
use toml;
use hyper::error::Error as HttpError;

#[derive(Debug)]
pub enum Kind {
    ChangeNotFound,
    PhaseNotFound,
    AuthenticationFailed,
    InternalServerError,
    NoMatchingCommand,
    NotOnABranch,
    CannotReviewSameBranch,
    FailedToExecute,
    PushFailed,
    BadGitOutputMatch,
    MissingMetadataVersion,
    BadMetadataVersionMatch,
    NoGitConfig,
    NoDeliveryConfig,
    NoBitbucketSCPConfig,
    NoGithubSCPConfig,
    OptionConstraint,
    UnknownProjectType,
    GitFailed,
    GitSetupFailed,
    ConfigParse,
    MissingConfig,
    MissingConfigFile,
    ConfigValidation,
    IoError,
    JsonError,
    JsonEncode,
    NoBuildCookbook,
    NoHomedir,
    ExpectedJsonString,
    BerksFailed,
    NoValidBuildCookbook,
    CopyFailed,
    MissingBuildCookbookName,
    SupermarketFailed,
    MoveFailed,
    RemoveFailed,
    TarFailed,
    MissingBuildCookbookField,
    ChefServerFailed,
    ChownFailed,
    ChefFailed,
    ChmodFailed,
    UnsupportedHttpMethod,
    HttpError(HttpError),
    UnsupportedProtocol,
    ApiError(hyper::status::StatusCode, Result<String, io::Error>),
    JsonParseError,
    TomlDecodeError,
    IntParseError,
    OpenFailed,
    NoToken,
    TokenExpired,
    NoEditor,
    MissingProjectConfig
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
            Kind::ChangeNotFound => "GET failed for specific change",
            Kind::PhaseNotFound => "Phase not implemented",
            Kind::NoMatchingCommand => "No command matches your arguments - likely unimplemented feature",
            Kind::NotOnABranch => "You must be on a branch",
            Kind::CannotReviewSameBranch => "You cannot target code for review from the same branch as the review is targeted for",
            Kind::FailedToExecute => "Tried to fork a process, and failed",
            Kind::PushFailed => "Git Push failed!",
            Kind::GitFailed => "Git command failed!",
            Kind::GitSetupFailed => "Setup failed; you have already set up delivery.",
            Kind::BadGitOutputMatch => "A line of git porcelain did not match!",
            Kind::BadMetadataVersionMatch => "Metadata version mismatch!",
            Kind::MissingMetadataVersion => "Missing a version entry into the metadata.rb",
            Kind::NoGitConfig => "Cannot find a .git/config file. Run 'git init' in your project root to initialize it.",
            Kind::NoDeliveryConfig => "Cannot find a .delivery/config.json file.",
            Kind::NoBitbucketSCPConfig => "Bitbucket Source Code Provider configuration not found; a Delivery administrator must first configure the link with Bitbucket",
            Kind::NoGithubSCPConfig => "Github Source Code Provider configuration not found; a Delivery administrator must first configure the link with Github",
            Kind::OptionConstraint => "Invalid option constraint",
            Kind::UnknownProjectType => "Unknown Project Type",
            Kind::ConfigParse => "Failed to parse the cli config file",
            Kind::MissingConfig => "A configuration value is missing",
            Kind::MissingConfigFile => "Could not find the configuration file.",
            Kind::ConfigValidation => "A required option is missing - use the command line options or 'delivery setup'",
            Kind::IoError => "An I/O Error occurred",
            Kind::JsonError => "A JSON Parser error occured",
            Kind::JsonEncode => "A JSON Encoding error occured",
            Kind::NoBuildCookbook => "No build_cookbook entry in .delivery/config.json",
            Kind::NoHomedir => "Cannot find a homedir",
            Kind::BerksFailed => "Berkshelf command failed",
            Kind::ExpectedJsonString => "Expected a JSON string",
            Kind::NoValidBuildCookbook => "Cannot find a valid build_cookbook entry in .delivery/config.json",
            Kind::MissingBuildCookbookName => "You must have a name field in you build_cookbook",
            Kind::CopyFailed => "Failed to copy files",
            Kind::SupermarketFailed => "Failed to download a cookbook from the supermarket",
            Kind::TarFailed => "Cannot untar a file",
            Kind::MoveFailed => "Cannot move a file",
            Kind::RemoveFailed => "Cannot remove a file or directory",
            Kind::MissingBuildCookbookField => "Missing a required field in your build_cookbook",
            Kind::ChefServerFailed => "Failed to download a cookbook from the Chef Server",
            Kind::ChownFailed => "Cannot set ownership to the dbuild user and group",
            Kind::ChefFailed => "Chef Client failed",
            Kind::ChmodFailed => "Cannot set permissions",
            Kind::UnsupportedHttpMethod => "Unsupported HTTP method",
            Kind::UnsupportedProtocol => "Unsupported protocol",
            Kind::HttpError(_) => "An HTTP Error occured",
            Kind::ApiError(_, _) => "An API Error occured",
            Kind::JsonParseError => "Attempted to parse invalid JSON",
            Kind::TomlDecodeError => "Attempted to decode invalid TOML",
            Kind::IntParseError => "Attempted to parse invalid Int",
            Kind::OpenFailed => "Open command failed",
            Kind::AuthenticationFailed => "Authentication failed",
            Kind::InternalServerError => "There was an internal error on the server. Check the logson the Delivery server.",
            Kind::NoToken => "Missing API token. Try `delivery token` to create one",
            Kind::TokenExpired => "The API token has expired. Try `delivery token` to generate a new one",
            Kind::NoEditor => "Environment variable EDITOR not set",
            Kind::MissingProjectConfig => "Unable to find .delivery/config.json in this directory or its parents"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            Kind::HttpError(ref e) => Some(e),
            Kind::ApiError(_, ref e) => {
                match *e {
                    Ok(_) => None,
                    Err(ref e) => Some(e)
                }
            },
            _ => None
        }
    }
}

impl fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<json::EncoderError> for DeliveryError {
    fn from(err: json::EncoderError) -> DeliveryError {
        DeliveryError{
            kind: Kind::JsonEncode,
            detail: Some(err.description().to_string())
        }
    }
}

impl From<json::DecoderError> for DeliveryError {
    fn from(err: json::DecoderError) -> DeliveryError {
        DeliveryError{
            kind: Kind::JsonParseError,
            detail: Some(format!("{}: {}", err.description().to_string(), err))
        }
    }
}

impl From<io::Error> for DeliveryError {
    fn from(err: io::Error) -> DeliveryError {
        DeliveryError{
            kind: Kind::IoError,
            detail: Some(format!("{}", err))
        }
    }
}

impl From<json::ParserError> for DeliveryError {
    fn from(err: json::ParserError) -> DeliveryError {
        DeliveryError{
            kind: Kind::JsonError,
            detail: Some(err.description().to_string())
        }
    }
}

impl From<HttpError> for DeliveryError {
    fn from(err: HttpError) -> DeliveryError {
        let detail = Some(err.description().to_string());
        DeliveryError{
            kind: Kind::HttpError(err),
            detail: detail
        }
    }
}

impl From<num::ParseIntError> for DeliveryError {
    fn from(err: num::ParseIntError) -> DeliveryError {
        let detail = Some(err.description().to_string());
        DeliveryError{
            kind: Kind::IntParseError,
            detail: detail
        }
    }
}

impl From<toml::DecodeError> for DeliveryError {
    fn from(err: toml::DecodeError) -> DeliveryError {
        DeliveryError{
            kind: Kind::TomlDecodeError,
            detail: Some(format!("{}: {}", err.description().to_string(), err))
        }
    }
}
