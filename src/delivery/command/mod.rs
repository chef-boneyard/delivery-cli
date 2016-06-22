pub mod lint;
pub mod syntax;

use utils;
use std::process::{Stdio};

pub fn run_chef_exec_command(exec_cmd: &str, args: &Vec<&str>) -> i32 {
    let mut gen = utils::make_command("chef");
    let output = gen.arg("exec")
        .arg(exec_cmd)
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
