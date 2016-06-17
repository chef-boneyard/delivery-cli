use cli::{for_arg, config_path_arg, u_e_s_o_args, value_of};
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "setup";

#[derive(Debug)]
pub struct SetupClapOptions<'n> {
    pub user: &'n str,
    pub server: &'n str,
    pub ent: &'n str,
    pub org: &'n str,
    pub path: &'n str,
    pub pipeline: &'n str,
}

impl<'n> Default for SetupClapOptions<'n> {
    fn default() -> Self {
        SetupClapOptions {
            user: "",
            server: "",
            ent: "",
            org: "",
            path: "",
            pipeline: "master",
        }
    }
}

impl<'n> SetupClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        SetupClapOptions {
            user: value_of(&matches, "user"),
            server: value_of(&matches, "server"),
            ent: value_of(&matches, "ent"),
            org: value_of(&matches, "org"),
            path: value_of(&matches, "config-path"),
            pipeline: value_of(&matches, "for"),
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Write a config file capturing specified options")
        .args(&vec![for_arg(), config_path_arg()])
        .args(&u_e_s_o_args())
}
