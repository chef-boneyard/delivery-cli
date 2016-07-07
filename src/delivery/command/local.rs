use cli::local::LocalClapOptions;
use types::{DeliveryResult, ExitCode};
use utils::say::{sayln, say};
use delivery_config::project::ProjectToml;
use project;

pub fn run(opts: LocalClapOptions) -> DeliveryResult<ExitCode> {
    sayln("green", "Chef Delivery");
    say("white", "Parsing ");
    say("yellow", ".delivery/project.toml");
    sayln("white", " file");
    let project_toml: ProjectToml = try!(
        ProjectToml::load_toml_file(project::project_path())
    );

    say("white", "Running ");
    say("magenta", &format!("{}", opts.phase));
    sayln("white", " Phase");
    sayln("green", &format!("Run here the right phase command: {:?}",
                    project_toml.local_phases));
    Ok(0)
}
