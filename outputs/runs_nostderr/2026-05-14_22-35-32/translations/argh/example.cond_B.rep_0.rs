use std::env;
use std::process::exit;
use std::collections::HashMap;

struct CmdLine {
    args: Vec<String>,
    flags: Vec<String>,
    params: HashMap<String, String>,
    pos_args: Vec<String>,
}

impl CmdLine {
    fn new() -> Self {
        CmdLine {
            args: Vec::new(),
            flags: Vec::new(),
            params: HashMap::new(),
            pos_args: Vec::new(),
        }
    }

    fn parse(&mut self, args: Vec<String>) {
        let mut args_iter = args.iter();
        while let Some(arg) = args_iter.next() {
            if arg.starts_with('-') {
                if let Some(next_arg) = args_iter.clone().next() {
                    if !next_arg.starts_with('-') {
                        self.params.insert(arg.clone(), next_arg.clone());
                        args_iter.next();
                    } else {
                        self.flags.push(arg.clone());
                    }
                } else {
                    self.flags.push(arg.clone());
                }
            } else {
                self.pos_args.push(arg.clone());
            }
        }
    }

    fn is_present(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string())
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    fn params(&self) -> &HashMap<String, String> {
        &self.params
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdl = CmdLine::new();
    cmdl.parse(args[1..].to_vec()); // Skipping the first argument which is the program name

    if cmdl.is_present("-v") {
        println!("Verbose, I am.");
    }

    println!("Positional args:");
    for pos_arg in cmdl.pos_args() {
        println!("\t{}", pos_arg);
    }

    println!("\nFlags:");
    for flag in cmdl.flags() {
        println!("\t{}", flag);
    }

    println!("\nParameters:");
    for (key, value) in cmdl.params() {
        println!("\t{} : {}", key, value);
    }

    exit(0);
}