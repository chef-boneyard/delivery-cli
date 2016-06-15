// Module for running, error handling, and UI outputting
// the lint command based on an input of arguments to it.
// Lint is a wrapper command around cookstyle.

use utils;
use std::process::{Stdio};

pub fn run(args: &Vec<&str>) -> i32 {
    let mut gen = utils::make_command("chef");
    let output = gen.arg("exec")
        .arg("cookstyle")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| { panic!("Unexpected error: Failed to execute process: {}", e) });

    let return_code = match output.status.code() {
        Some(code) => code,
        _ => 1
    };
    return return_code;
}

