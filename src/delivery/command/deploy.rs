use command::wrap_kitchen_command;

pub fn run(args: &Vec<&str>) -> i32 {
    return wrap_kitchen_command(args, "kitchen converge", "USAGE:\n    delivery local deploy [INSTANCE|REGEXP|all] # Change instance state to converge. Use a provisioner to configure one or more instances")
}

