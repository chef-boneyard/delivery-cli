pub mod cleanup;
pub mod deploy;
pub mod lint;
pub mod provision;
pub mod smoke;
pub mod syntax;
pub mod unit;

use utils;
use std::process::{Stdio};

pub fn run_chef_exec_command(exec_cmd: &str, args: &Vec<&str>) -> i32 {
    // Split args on whitespace, so if we pass in the exec_cmd:
    // "exec kitchen"
    // we can convert it to a &Vec<&str> and pass that to .args(),
    // so that ARGV properly looks like ["kitchen", "create", ..]
    // instead of ["kitchen create", ...]
    let split_cmd_arg_vec = exec_cmd.split_whitespace().collect::<Vec<&str>>();
    let mut gen = utils::make_command("chef");
    let output = gen.arg("exec")
        .args(&split_cmd_arg_vec)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| { panic!("Unexpected error: Failed to execute process: {}", e) });

    let return_code = match output.status.code() {
        Some(code) => code,
        _ => 1
    };
    return return_code
}

pub fn wrap_kitchen_command(args: &Vec<&str>, kitchen_cmd: &str, usage: &str) -> i32 {
    if !args.is_empty() {
        match args[0].as_ref() {
            // kitchen subcommands don't respond to --help, so let's return something useful.
            "--help" => {
                // Should be string stolen from kitchen --help
                println!("{}", usage);
                return 0
            },
            _ => return run_chef_exec_command(kitchen_cmd, args)
        }
    } else {
        return run_chef_exec_command(kitchen_cmd, args)
    }
}
