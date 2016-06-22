use command::wrap_kitchen_command;

pub fn run(args: &Vec<&str>) -> i32 {
    return wrap_kitchen_command(args, "kitchen create", "USAGE:\n    delivery local provision [INSTANCE|REGEXP|all] # Change instance state to create. Start one or more instances")
}

