use cli::init::InitClapOptions;

// TODO delete
use cli::load_config;

use project;
use utils::cwd;
use utils::say::{sayln};
use errors::{DeliveryError, Kind};

// Takes in fully parsed and defaulted init clap args,
// executes init code, handles errors, as well as UI output.
//
// Returns an integer exit code, handling all errors it knows how to
// and panicing on unexpected errors.
pub fn run(init_opts: InitClapOptions) -> Result<(), DeliveryError> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&cwd()));
    let final_proj = try!(project::project_or_from_cwd(init_opts.project));
    config = config.set_user(init_opts.user)
        .set_server(init_opts.server)
        .set_enterprise(init_opts.ent)
        .set_organization(init_opts.org)
        .set_project(&final_proj)
        .set_pipeline(init_opts.pipeline)
        .set_generator(init_opts.generator)
        .set_config_json(init_opts.config_json);
    let branch = try!(config.pipeline());
    if !init_opts.github_org_name.is_empty() && !init_opts.bitbucket_project_key.is_empty() {
        return Err(DeliveryError{ kind: Kind::OptionConstraint, detail: Some(format!("Please \
        specify just one Source Code Provider: delivery(default), github or bitbucket.")) })
    }
    let mut scp: Option<project::SourceCodeProvider> = None;
    if !init_opts.github_org_name.is_empty() {
        debug!("init github: GitRepo:{:?}, GitOrg:{:?}, Branch:{:?}, SSL:{:?}",
               init_opts.repo_name, init_opts.github_org_name, branch, init_opts.no_v_ssl);
        scp = Some(try!(project::SourceCodeProvider::new("github", &init_opts.repo_name,
                                                         &init_opts.github_org_name, &branch,
                                                         init_opts.no_v_ssl)));
    } else if !init_opts.bitbucket_project_key.is_empty() {
        debug!("init bitbucket: BitRepo:{:?}, BitProjKey:{:?}, Branch:{:?}",
               init_opts.repo_name, init_opts.bitbucket_project_key, branch);
        scp = Some(try!(project::SourceCodeProvider::new("bitbucket", &init_opts.repo_name,
                                                         &init_opts.bitbucket_project_key,
                                                         &branch, true)));
    }
    project::init(config, &init_opts.no_open, &init_opts.skip_build_cookbook, &init_opts.local, scp)
}
