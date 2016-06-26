use cli::init::InitClapOptions;

// TODO delete
use cli::load_config;

use project;
use utils::cwd;
use utils::say::{sayln};

// Takes in fully parsed and defaulted init clap args,
// executes init code, handles errors, as well as UI output.
//
// Returns an integer exit code, handling all errors it knows how to
// and panicing on unexpected errors.
pub fn run(init_opts: InitClapOptions) -> i32 {
    sayln("green", "Chef Delivery");
    let mut config = load_config(&cwd()).unwrap();
    let final_proj = project::project_or_from_cwd(init_opts.project).unwrap();
    config = config.set_user(init_opts.user)
        .set_server(init_opts.server)
        .set_enterprise(init_opts.ent)
        .set_organization(init_opts.org)
        .set_project(&final_proj)
        .set_pipeline(init_opts.pipeline)
        .set_generator(init_opts.generator)
        .set_config_json(init_opts.config_json);
    let branch = config.pipeline().unwrap();

    if !init_opts.github_org_name.is_empty() && !init_opts.bitbucket_project_key.is_empty() {
        sayln("red", "Please specify just one Source Code Provider: delivery(default), github or bitbucket.");
        return 1;
    }

    let mut scp: Option<project::SourceCodeProvider> = None;
    if !init_opts.github_org_name.is_empty() {
        scp = Some(
            project::SourceCodeProvider::new("github", &init_opts.repo_name,
                                             &init_opts.github_org_name, &branch,
                                             init_opts.no_v_ssl).unwrap()
        );
    } else if !init_opts.bitbucket_project_key.is_empty() {
        scp = Some(
            project::SourceCodeProvider::new("bitbucket", &init_opts.repo_name,
                                             &init_opts.bitbucket_project_key,
                                             &branch, true).unwrap()
        );
    }
    project::init(config, &init_opts.no_open, &init_opts.skip_build_cookbook, &init_opts.local, scp).unwrap();
    return 0;
}
