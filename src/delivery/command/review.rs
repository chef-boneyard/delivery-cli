use cli::review::ReviewClapOptions;
use delivery_config::DeliveryConfig;
use cli::load_config;
use config::Config;
use project;
use utils;
use utils::say::{sayln, say};
use errors::DeliveryError;
use types::{DeliveryResult, ExitCode};
use cookbook;
use git::{self, ReviewResult};
use http;

pub fn run(review_opts: ReviewClapOptions) -> DeliveryResult<ExitCode> {
    sayln("green", "Chef Delivery");
    let mut config = try!(load_config(&utils::cwd()));
    config = config.set_pipeline(review_opts.pipeline);
    let target = validate!(config, pipeline);
    let project_root = try!(project::root_dir(&utils::cwd()));
    try!(DeliveryConfig::validate_config_file(&project_root));

    if review_opts.auto_bump {
        config.auto_bump = Some(review_opts.auto_bump.clone())
    }

    if let Some(should_bump) = config.auto_bump {
        if should_bump {
            try!(cookbook::bump_version(&project_root, &target))
        }
    }

    let head = try!(git::get_head());
    say("white", "Review for change ");
    say("yellow", &head);
    say("white", " targeted for pipeline ");
    sayln("magenta", &target);
    let review = try!(project::review(&target, &head));

    if review_opts.edit {
        let project = try!(project::project_from_cwd());
        config = config.set_pipeline(review_opts.pipeline)
            .set_project(&project);
        try!(edit_change(&config, &review));
    }

    for line in review.messages.iter() {
        sayln("white", line);
    }

    match try!(project::handle_review_result(&review, &review_opts.no_open)) {
        Some(url) => {sayln("magenta", &url)},
        None => {}
    }
    Ok(0)
}

fn edit_change(config: &Config,
               review: &ReviewResult) -> Result<(), DeliveryError> {
    let proj = try!(config.project());
    match review.change_id {
        Some(ref change_id) => {
            let change0 = try!(http::change::get(&config, &change_id));
            let text0 = format!("{}\n\n{}\n",
                                change0.title, change0.description);
            let text1 = try!(utils::open::edit_str(&proj, &text0));
            let change1 = try!(http::change::Description::parse_text(&text1));
            Ok(try!(http::change::set(&config, &change_id, &change1)))
        },
        None => Ok(())
    }
}
