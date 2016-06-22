use command::run_chef_exec_command;

pub fn run(args: &Vec<&str>) -> i32 {
    return run_chef_exec_command("foodcritic", args)
}
