use cli::{config_path_arg, value_of, u_e_s_o_args};
use clap::{Arg, App, SubCommand, ArgMatches};

pub const SUBCOMMAND_NAME: &'static str = "api";

#[derive(Debug)]
pub struct ApiClapOptions<'n> {
    pub method: &'n str,
    pub path: &'n str,
    pub data: &'n str,
    pub server: &'n str,
    pub api_port: &'n str,
    pub ent: &'n str,
    pub user: &'n str,
}
impl<'n> Default for ApiClapOptions<'n> {
    fn default() -> Self {
        ApiClapOptions {
            method: "",
            path: "",
            data: "",
            server: "",
            api_port: "",
            ent: "",
            user: ""
        }
    }
}

impl<'n> ApiClapOptions<'n> {
    pub fn new(matches: &'n ArgMatches<'n>) -> Self {
        ApiClapOptions {
            method: value_of(&matches, "method"),
            path: value_of(&matches, "path"),
            data: value_of(&matches, "data"),
            server: value_of(&matches, "server"),
            api_port: value_of(&matches, "api-port"),
            ent: value_of(&matches, "ent"),
            user: value_of(&matches, "user")
        }
    }
}

pub fn clap_subcommand<'c>() -> App<'c, 'c> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("Helper to call Delivery's HTTP API")
        .args(&vec![config_path_arg()])
        .arg(Arg::from_usage("<method> 'HTTP method for the request'")
             .takes_value(false)
             .possible_values(&["get", "put", "post", "delete"]))
        .args_from_usage(
             "<path> 'Path for rqeuest URL'
             --api-port=[api-port] 'Port for Delivery server'")
        .arg(Arg::with_name("data")
             .long("data")
             .short("d")
             .help("Data to send for PUT/POST request")
             .takes_value(true)
             .multiple(false)
             .number_of_values(1)
             .use_delimiter(false))
        .args(&u_e_s_o_args())
}
