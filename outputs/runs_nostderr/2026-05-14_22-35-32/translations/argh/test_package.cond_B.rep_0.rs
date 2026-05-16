use std::collections::HashMap;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmdl = parse_args(&args);

    if cmdl.get("-v").is_some() {
        println!("Verbose, I am.");
    }
}

fn parse_args(args: &[String]) -> HashMap<String, Option<String>> {
    let mut args_map = HashMap::new();
    let mut i = 1; // Skip the first argument which is the program name

    while i < args.len() {
        if args[i].starts_with('-') {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                args_map.insert(args[i].clone(), Some(args[i + 1].clone()));
                i += 1; // increment to skip next argument as it has been paired with the flag
            } else {
                args_map.insert(args[i].clone(), None);
            }
        }
        i += 1;
    }

    args_map
}