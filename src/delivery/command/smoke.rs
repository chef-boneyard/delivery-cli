use command::wrap_kitchen_command;

pub fn run(args: &Vec<&str>) -> i32 {
    return wrap_kitchen_command(args, "kitchen verify", "USAGE:\n    delivery local smoke [INSTANCE|REGEXP|all] # Change instance state to verify. Run automated tests on one or more instances")
}
