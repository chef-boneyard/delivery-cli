use cli::{for_arg, patchset_arg, value_of};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "checkout";

#[derive(Debug)]
pub struct CheckoutClapOptions<'n> {
    pub pipeline: &'n str,
    pub change: &'n str,
    pub patchset: &'n str,
}
impl<'n> Default for CheckoutClapOptions<'n> {
    fn default() -> Self {
        CheckoutClapOptions {
            pipeline: "master",
            change: "",
            patchset: "",
        }
    }
}

impl<'n> CheckoutClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        CheckoutClapOptions {
            pipeline: value_of(&matches, "for"),
            change: value_of(&matches, "change"),
            patchset: value_of(&matches, "patchset"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Create a local branch tracking an in-progress change")
        .args(&vec![for_arg(), patchset_arg()])
        .args_from_usage("<change> 'Name of the feature branch to checkout'")
}
