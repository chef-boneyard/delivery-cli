#![allow(unstable)]
use rustc_serialize::json;
use job::workspace::Workspace;
use job::change::Change;

#[derive(RustcEncodable)]
pub struct Top {
    pub workspace: Workspace,
    pub change: Change,
    pub config: json::Json
}

#[derive(RustcEncodable)]
pub struct DNA {
    pub delivery: Top,
}

