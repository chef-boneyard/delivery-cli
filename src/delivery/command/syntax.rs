use command::run_chef_exec_command;

pub fn run(args: &Vec<&str>) -> i32 {
    // If no additional args were passed, assume the cookbook is in .
    if args.is_empty() {
        return run_chef_exec_command("foodcritic . --exclude spec -f any", args)
    } else {
        return run_chef_exec_command("foodcritic", args)
    }
}
