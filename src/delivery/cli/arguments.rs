//
// Copyright:: Copyright (c) 2016 Chef Software, Inc.
// License:: Apache License, Version 2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use clap::{Arg, ArgMatches};

pub fn value_of<'a>(matches: &'a ArgMatches, key: &str) -> &'a str {
    matches.value_of(key).unwrap_or("")
}

macro_rules! make_arg_vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(Arg::from_usage($x));
            )*
            temp_vec
        }
    };
}

macro_rules! fn_arg {
    ( $fn_name:ident, $usage:expr ) => (
        pub fn $fn_name<'a>() -> Arg<'a, 'a> {
            Arg::from_usage($usage)
        }
    )
}

pub fn u_e_s_o_args<'a>() -> Vec<Arg<'a, 'a>> {
    make_arg_vec![
        "-u --user=[user] 'User name for Delivery authentication'",
        "-e --ent=[ent] 'The enterprise in which the project lives'",
        "-o --org=[org] 'The organization in which the project lives'",
        server_arg_str()]
}

// Defines all the options shared between commands that
// are project specific (i.e. must be run in the context of a git repo).
pub fn project_specific_args<'a>() -> Vec<Arg<'a, 'a>> {
    make_arg_vec![
        "--fips 'Runs command in FIPS mode. This proxies all git traffic through Stunnel FIPS encryption'",
        "--fips-git-port=[port] 'The port Stunnel listens locally on when proxying git traffic'",
        "--fips-custom-cert-filename=[cert] 'The path to a pem file that contains a self-signed certificate or certificate chain. Use this setting only when Automate server has a custom certificate authority or a self-signed certificate. Please see the Automate FIPS docs for more details.'"
    ]
}

pub fn scp_args<'a>() -> Vec<Arg<'a, 'a>> {
    make_arg_vec![
        "--bitbucket=[project-key] 'Use a Bitbucket repository for Code Review with the provided Project Key'",
        "--github=[org-name] 'Use a Github repository for Code Review with the provided Organization'",
        "-r --repo-name=[repo-name] 'Source code provider repository name'",
        "--no-verify-ssl 'Do not use SSL verification. [Github]'"]
}

pub fn pipeline_arg<'a>() -> Vec<Arg<'a, 'a>> {
    vec![Arg::from_usage(
            "-f --pipeline=[pipeline] 'Target pipeline for change (default: master)'"
        ).visible_alias("for")]
}

pub fn server_arg_str<'a>() -> &'a str {
    "-s --server=[server] 'The Automate server address'"
}

pub fn api_port_arg_str<'a>() -> &'a str {
    "--api-port=[api-port] 'Port for Automate server'"
}

fn_arg!(server_arg, server_arg_str());

fn_arg!(api_port_arg, api_port_arg_str());

fn_arg!(config_project_arg,
       "-c --config-json=[config-json] 'Path of a custom config.json file'");

fn_arg!(patchset_arg,
       "-P --patchset=[patchset] 'A patchset number (default: latest)'");

fn_arg!(project_arg,
       "-p --project=[project] 'The project name'");

fn_arg!(config_path_arg,
        "--config-path=[dir] 'Directory to read/write your config file \
         (cli.toml) from'");

fn_arg!(local_arg, "-l --local 'Operate without a Delivery server'");

fn_arg!(no_open_arg, "-n --no-open 'Do not open the change in a browser'");

fn_arg!(auto_bump, "-a --auto-bump 'Automatic cookbook version bump'");

fn_arg!(no_spinner_arg, "--no-spinner 'Disable the spinner'");

fn_arg!(no_color_arg, "--no-color 'Disable colors in terminal'");

fn_arg!(non_interactive_arg, "--non-interactive 'Disable command line interactions'");

#[cfg(test)]
mod tests {
    use cli;
    use super::value_of;

    #[test]
    fn test_value_of_trait() {
        let build_version = format!("{} {}", cli::version(), cli::build_git_sha());

        let matches = cli::make_app(&build_version).get_matches_from(
            vec!["delivery", "checkout", "branch", "--for", "griffindor"]
        );
        let cmd_matches = matches.subcommand_matches(cli::checkout::SUBCOMMAND_NAME).unwrap();
        // A simple argument
        assert_eq!("griffindor", value_of(&cmd_matches, "pipeline"));
        assert_eq!("", value_of(&cmd_matches, "not_for"));

        let matches = cli::make_app(&build_version).get_matches_from(
            vec!["delivery", "checkout", "branch", "--pipeline", "hufflepuff"]
        );
        let cmd_matches = matches.subcommand_matches(cli::checkout::SUBCOMMAND_NAME).unwrap();
        // A simple argument
        assert_eq!("hufflepuff", value_of(&cmd_matches, "pipeline"));
        assert_eq!("", value_of(&cmd_matches, "not_pipeline"));
    }
}
