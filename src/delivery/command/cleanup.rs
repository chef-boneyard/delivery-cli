use command::wrap_kitchen_command;

pub fn run(args: &Vec<&str>) -> i32 {
    return wrap_kitchen_command(args, "kitchen destroy", "USAGE:\n    delivery local cleanup [INSTANCE|REGEXP|all] # Change instance state to destroy. Delete all information for one or more instances")
}
