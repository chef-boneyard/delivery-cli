use uuid::Uuid;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Change {
    pub enterprise: String,
    pub organization: String,
    pub project: String,
    pub pipeline: String,
    pub change_id: Uuid,
    pub patchset_number: f64,
    pub stage: String,
    pub stage_run_id: f64,
    pub phase: String,
    pub phase_run_id: f64,
    pub git_url: String,
    pub sha: String,
    pub patchset_branch: String,
    pub delivery_api_url: Option<String>,
    pub delivery_data_url: Option<String>,
    pub delivery_change_url: Option<String>,
    pub log_level: String,
    pub token: Option<String>
}

