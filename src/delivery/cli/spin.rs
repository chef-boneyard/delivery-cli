use cli::value_of;
use clap::{App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "spin";

#[derive(Debug)]
pub struct SpinClapOptions {
    pub time: u64,
}
impl Default for SpinClapOptions {
    fn default() -> Self {
        SpinClapOptions {
            time: 5,
        }
    }
}

impl SpinClapOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let t = value_of(&matches, "time");
        if t.is_empty() {
            Default::default()
        } else {
            SpinClapOptions {
                time: t.parse::<u64>().unwrap(),
            }
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("test the spinner")
        .args_from_usage("-t --time=[TIME] 'How many seconds to spin. default:5'")
}
