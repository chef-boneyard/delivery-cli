use clap::{App, SubCommand, ArgMatches, Arg};
use delivery_config::project::Phase;
use cli::value_of;

pub const SUBCOMMAND_NAME: &'static str = "local";

#[derive(Debug)]
pub struct LocalClapOptions {
    pub phase: Option<Phase>
}

impl Default for LocalClapOptions {
    fn default() -> Self {
        LocalClapOptions { phase: None }
    }
}

impl LocalClapOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let phase = match value_of(matches, "phase") {
            "unit" => Some(Phase::Unit),
            "lint" => Some(Phase::Lint),
            "syntax" => Some(Phase::Syntax),
            "provision" => Some(Phase::Provision),
            "deploy" => Some(Phase::Deploy),
            "smoke" => Some(Phase::Smoke),
            "cleanup" => Some(Phase::Cleanup),
            _ => None
        };

        LocalClapOptions { phase: phase }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Run Delivery phases on your local workstation.")
        .arg(Arg::from_usage("<phase> 'Delivery phase to execute'")
             .takes_value(false)
             .possible_values(&["unit", "lint", "syntax", "provision",
                                "deploy", "smoke", "cleanup"]))
}
