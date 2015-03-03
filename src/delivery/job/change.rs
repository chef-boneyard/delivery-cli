#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Change {
    pub enterprise: String,
    pub organization: String,
    pub project: String,
    pub pipeline: String,
    pub change_id: String,
    pub patchset_number: String,
    pub stage: String,
    pub phase: String,
    pub git_url: String,
    pub sha: String,
    pub patchset_branch: String,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct BuilderCompat {
    pub workspace: String,
    pub repo: String,
    pub cache: String,
    pub build_id: String,
    pub build_user: String
}


