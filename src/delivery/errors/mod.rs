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

use serde_json;
use std::error::{self, Error};
use std::num;
use std::io;
use std::fmt;
use std::string;
use hyper;
use toml;
use types::ExitCode;
use hyper::error::Error as HttpError;

#[derive(Debug)]
pub enum Kind {
    ChangeNotFound,
    PhaseNotFound,
    PhaseFailed(ExitCode),
    LocalPhasesNotFound,
    AuthenticationFailed,
    ForbiddenRequest,
    InternalServerError,
    EndpointNotFound,
    NoMatchingCommand,
    ClapArgAliasOverlap,
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
    ProjectSCPNameMismatch,
    OptionConstraint,
    UnknownProjectType,
    ProjectNotFound(String),
    UserNotFound(String),
    GitFailed,
    UnauthorizedAction,
    MissingSshPubKey,
    EmptyGitCommit,
    GitSetupFailed,
    ConfigParse,
    DeliveryConfigParse,
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
    CloneFailed,
    TarFailed,
    MissingBuildCookbookField,
    ChefServerFailed,
    ChefdkGenerateFailed,
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
    MissingProjectConfig,
    MissingRequiredConfigOption,
    FipsNotSupportedForChefDKPlatform,
    AutomateNginxCertFetchFailed,
    FromUtf8Error,
    BranchNotFoundOnDeliveryRemote,
}

#[derive(Debug)]
pub struct DeliveryError {
    pub kind: Kind,
    pub detail: Option<String>,
}

impl DeliveryError {
    /// Constructor
    ///
    /// Use this method to create a DeliveryError struct.
    ///
    /// # Example:
    ///
    /// ```rust
    /// use delivery::errors::DeliveryError;
    /// use delivery::errors::Kind::IoError;
    ///
    /// let e = DeliveryError::throw(IoError, None);
    /// assert!(e.detail.is_none());
    /// ```
    pub fn throw(kind: Kind, detail: Option<String>) -> Self {
        DeliveryError { kind: kind, detail: detail }
    }

    pub fn detail(&self) -> Option<String> {
        self.detail.clone()
    }
}

impl error::Error for DeliveryError {
    fn description(&self) -> &str {
        match self.kind {
            Kind::ChangeNotFound => "GET failed for specific change",
            Kind::PhaseNotFound => "Phase not implemented",
            Kind::PhaseFailed(_) => "Phase failed!",
            Kind::LocalPhasesNotFound => "LocalPhases tag not found",
            Kind::NoMatchingCommand => "No command matches your arguments - likely unimplemented feature",
            Kind::ClapArgAliasOverlap => "There was an argument/alias overlap.",
            Kind::NotOnABranch => "You must be on a branch",
            Kind::CannotReviewSameBranch => "You cannot target code for review from the same branch as the review is targeted for",
            Kind::FailedToExecute => "Tried to fork a process, and failed",
            Kind::PushFailed => "Git Push failed!",
            Kind::GitFailed => "Git command failed!",
            Kind::UnauthorizedAction => "You are not authorized to perform this action.",
            Kind::MissingSshPubKey => "Missing SSH public key on the server side.",
            Kind::EmptyGitCommit => "Nothing to commit, working directory clean",
            Kind::GitSetupFailed => "Setup failed; you have already set up delivery.",
            Kind::BadGitOutputMatch => "A line of git porcelain did not match!",
            Kind::BadMetadataVersionMatch => "Metadata version mismatch!",
            Kind::MissingMetadataVersion => "Missing a version entry into the metadata.rb",
            Kind::NoGitConfig => "Cannot find a .git/config file. Run 'git init' in your project root to initialize it.",
            Kind::NoDeliveryConfig => "Cannot find a .delivery/config.json file.",
            Kind::NoBitbucketSCPConfig => "Bitbucket Source Code Provider configuration not found; a Delivery administrator must first configure the link with Bitbucket",
            Kind::NoGithubSCPConfig => "Github Source Code Provider configuration not found; a Delivery administrator must first configure the link with Github",
            Kind::ProjectSCPNameMismatch => "Project and repository name mismatch.",
            Kind::OptionConstraint => "Invalid option constraint",
            Kind::UnknownProjectType => "Unknown Project Type",
            Kind::ProjectNotFound(_) => "Project Not Found!",
            Kind::UserNotFound(_) => "User Not Found!",
            Kind::ConfigParse => "Failed to parse the cli config file",
            Kind::DeliveryConfigParse => "Unable to parse the config.json file.",
            Kind::MissingConfig => "A configuration value is missing",
            Kind::MissingConfigFile => "Could not find the configuration file.",
            Kind::ConfigValidation => "A required option is missing - use the command line options or 'delivery setup'",
            Kind::IoError => "An I/O Error occurred",
            Kind::JsonError => "A JSON Parser error occurred",
            Kind::JsonEncode => "A JSON Encoding error occurred",
            Kind::NoBuildCookbook => "No valid build_cookbook entry in .delivery/config.json",
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
            Kind::CloneFailed => "Unable to clone project.",
            Kind::MissingBuildCookbookField => "Missing a required field in your build_cookbook",
            Kind::ChefServerFailed => "Failed to download a cookbook from the Chef Server",
            Kind::ChefdkGenerateFailed => "Failed to execute 'chef generate'",
            Kind::ChownFailed => "Cannot set ownership to the dbuild user and group",
            Kind::ChefFailed => "Chef Client failed",
            Kind::ChmodFailed => "Cannot set permissions",
            Kind::UnsupportedHttpMethod => "Unsupported HTTP method",
            Kind::UnsupportedProtocol => "Unsupported protocol",
            Kind::HttpError(_) => "An HTTP Error occurred",
            Kind::ApiError(_, _) => "An API Error occurred",
            Kind::JsonParseError => "Attempted to parse invalid JSON",
            Kind::TomlDecodeError => "Attempted to decode invalid TOML",
            Kind::IntParseError => "Attempted to parse invalid Int",
            Kind::OpenFailed => "Open command failed",
            Kind::AuthenticationFailed => "401: Authentication failed",
            Kind::ForbiddenRequest => "403: Unauthorized request",
            Kind::InternalServerError => "500: There was an internal error on the server.\nCheck the logs on the Automate server.",
            Kind::EndpointNotFound => "404: Endpoint not found!",
            Kind::NoToken => "Missing API token. Try `delivery token` to create one",
            Kind::TokenExpired => "The API token has expired. Try `delivery token` to generate a new one",
            Kind::NoEditor => "Environment variable EDITOR not set",
            Kind::MissingProjectConfig => "Unable to find .delivery/config.json in this directory or its parents",
            Kind::MissingRequiredConfigOption => "A required config option was not set. Please specify in your cli.toml.",
            Kind::FipsNotSupportedForChefDKPlatform => "The ChefDK for your platform does not support FIPS mode.\nRHEL and Windows are the currently supported FIPS platforms for ChefDK.",
            Kind::AutomateNginxCertFetchFailed => "Fetching the Automate certificate failed. The automate certificate is required for FIPS mode. Please make sure you can connect to your Automate server.",
            Kind::FromUtf8Error => "Failed to convert bytes from Utf8 into a string.",
            Kind::BranchNotFoundOnDeliveryRemote => "Could not find specified branch on the delivery remote.",
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
        let msg = match self.kind {
            Kind::PhaseFailed(ref e) => format!("Phase failed with exit code ({})!", e),
            Kind::ProjectNotFound(ref e) => format!("The project '{}' was not found.", e),
            Kind::UserNotFound(ref e) => format!("The user '{}' was not found.", e),
            _ => self.description().to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl From<serde_json::Error> for DeliveryError {
    fn from(err: serde_json::Error) -> DeliveryError {
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

impl From<toml::de::Error> for DeliveryError {
    fn from(err: toml::de::Error) -> DeliveryError {
        DeliveryError{
            kind: Kind::TomlDecodeError,
            detail: Some(format!("{}: {}", err.description().to_string(), err))
        }
    }
}

impl From<toml::ser::Error> for DeliveryError {
    fn from(err: toml::ser::Error) -> DeliveryError {
        DeliveryError{
            kind: Kind::TomlDecodeError,
            detail: Some(format!("{}: {}", err.description().to_string(), err))
        }
    }
}

impl From<string::FromUtf8Error> for DeliveryError {
    fn from(err: string::FromUtf8Error) -> DeliveryError {
        let detail = Some(err.description().to_string());
        DeliveryError{
            kind: Kind::FromUtf8Error,
            detail: detail
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::DeliveryError;
    pub use super::Kind::{EndpointNotFound, JsonError};

    mod constructor {
        #[test]
        fn throw_without_detail() {
            let e = super::DeliveryError::throw(super::EndpointNotFound, None);
            assert!(e.detail.is_none());
            assert_enum!(e.kind, super::EndpointNotFound);
        }

        #[test]
        fn throw_with_detail() {
            let msg = "Your json looks funcky".to_string();
            // We clone it because we actually take ownership of the message
            let e = super::DeliveryError::throw(super::JsonError, Some(msg.clone()));
            assert!(e.detail.is_some());
            assert_eq!(e.detail.unwrap(), msg);
            assert_enum!(e.kind, super::JsonError);
        }
    }
}
