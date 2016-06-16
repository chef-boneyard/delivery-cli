use cli::{for_arg, patchset_arg, value_of};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "diff";

#[derive(Debug)]
pub struct DiffClapOptions<'n> {
    pub change: &'n str,
    pub patchset: &'n str,
    pub pipeline: &'n str,
    pub local: bool,
}
impl<'n> Default for DiffClapOptions<'n> {
    fn default() -> Self {
        DiffClapOptions {
            change: "",
            patchset: "",
            pipeline: "master",
            local: false,
        }
    }
}

impl<'n> DiffClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        DiffClapOptions {
            change: value_of(&matches, "change"),
            patchset: value_of(&matches, "patchset"),
            pipeline: value_of(&matches, "for"),
            local: matches.is_present("local"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Display diff for a change")
        .args(&vec![for_arg(), patchset_arg()])
        .args_from_usage(
            "<change> 'Name of the feature branch to compare'
            -l --local \
            'Diff against the local branch HEAD'")
}
