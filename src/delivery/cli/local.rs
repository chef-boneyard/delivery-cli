use clap::{App, SubCommand, ArgMatches, Arg};
use cli::value_of;

pub const SUBCOMMAND_NAME: &'static str = "local";

#[derive(Debug)]
pub struct LocalClapOptions<'n> {
    pub phase: &'n str
}

impl<'n> Default for LocalClapOptions<'n> {
    fn default() -> Self {
        LocalClapOptions { phase: "" }
    }
}

impl<'n> LocalClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        LocalClapOptions {
            phase: value_of(matches, "phase")
        }
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
