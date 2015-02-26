use rustc_serialize::json;
use job::workspace::Workspace;
use job::change::{Change, BuilderCompat};

#[derive(RustcEncodable)]
pub struct Top {
    pub workspace: Workspace,
    pub change: Change,
    pub config: json::Json
}

#[derive(RustcEncodable)]
pub struct DNA {
    pub delivery: Top,
    pub delivery_builder: BuilderCompat
}

