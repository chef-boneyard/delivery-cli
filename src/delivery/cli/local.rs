use clap::{App, SubCommand, ArgMatches, AppSettings};
use std::env;
use std::process;
use utils::say::sayln;
use errors::DeliveryError;

// Implemented sub-commands. Should handle everything after args have
// been parsed, including running the command, error handling, and UI outputting.
use command::deploy;
use command::lint;
use command::provision;
use command::syntax;
use command::unit;

// Local subcommand is for wrapping external commands and running
// the locally.
pub const SUBCOMMAND_NAME: &'static str = "local";

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .template(
            // Please keep alphabetized
            "{bin}\n{about}\n\n{usage}\n\nSUBCOMMANDS: \
             \n    deploy \
             \n    lint \
             \n    provision \
             \n    syntax \
             \n    unit"
        )
        // Use custom usage because gloabl flags will break parsing for this command
        .usage("delivery local <SUBCOMMAND> [SUBCOMMAND_FLAGS]")
        .about("Run Delivery phases on your local workstation.")
        .setting(AppSettings::AllowExternalSubcommands)
}

pub fn parse_clap_matches(global_matches: &ArgMatches) -> Result<(), DeliveryError> {
    match global_matches.subcommand() {
        // Matches any `delivery local <any_subcommand>`.
        (external, Some(sub_matches)) => {
            // This will get all args following <any_subcommand> from above match in an array, so:
            // `delivery local lint --lol fun hehe`
            // Would return:
            // ["--lol", "fun", "hehe"]
            // post_subcommand_args: Vec<&str>
            let post_subcommand_args: Vec<&str> = match sub_matches.values_of(external) {
                Some(values) => values.collect(),
                None => Vec::new()
            };

            // Unfortunately, if you use AppSettings::AllowExternalSubcommands,
            // clap does not actually capture what the original subcommand to local was.
            // However, it is the only way I found to allow arbitary arguments along in clap,
            // so we will just validate the subcommand to local directly.
            let args: Vec<_> = env::args().collect();

            // Match the third arg of `delivery local <any_subcommand>`.
            match args[2].as_ref() {
                "deploy" => {
                    process::exit(deploy::run(&post_subcommand_args))
                },
                "lint" => {
                    process::exit(lint::run(&post_subcommand_args))
                },
                "provision" => {
                    process::exit(provision::run(&post_subcommand_args))
                }
                "syntax" => {
                    process::exit(syntax::run(&post_subcommand_args))
                },
                "unit" => {
                    process::exit(unit::run(&post_subcommand_args))
                }
                unknown => {
                    sayln("red", &format!("You passed subcommand '{}' to 'delivery {}'.", unknown, SUBCOMMAND_NAME));
                    sayln("red", &format!("'{}' is not a valid subcommand for 'delivery {}'.", unknown, SUBCOMMAND_NAME));
                    sayln("red", &format!("To see valid subcommands, please run 'delivery {} --help'.", SUBCOMMAND_NAME));
                    process::exit(1)
                }
            }
        },
        _ => {
            sayln("red", &format!("You did not pass a subcommand to 'delivery {}'.", SUBCOMMAND_NAME));
            sayln("red", &format!("To see valid subcommands, please run 'delivery {} --help'.", SUBCOMMAND_NAME));
            process::exit(1)
        }
    }
    Ok(())
}
