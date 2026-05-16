use std::env;
use std::process::exit;

struct Argh {
    // Since we don't have an exact equivalent to argh::parser in Rust,
    // we need to provide our own definitions and implementations.
    positional_args: Vec<String>,
    flags: Vec<String>,
    params: Vec<(String, String)>,
}

impl Argh {
    fn new() -> Argh {
        Argh {
            positional_args: Vec::new(),
            flags: Vec::new(),
            params: Vec::new(),
        }
    }

    fn parse(&mut self, args: Vec<String>) {
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg.starts_with("-") {
                // Consider a flag
                if i + 1 < args.len() && !args[i + 1].starts_with("-") {
                    // Consider a parameter if the next argument is not a flag
                    self.params
                        .push((arg.clone(), args[i + 1].clone()));
                    i += 1; // Skip parameter value
                } else {
                    // Otherwise, it's just a flag
                    self.flags.push(arg.clone());
                }
            } else {
                // Positional argument
                self.positional_args.push(arg.clone());
            }
            i += 1;
        }
    }

    fn is_flag_set(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string())
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.positional_args
    }

    fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    fn params(&self) -> &Vec<(String, String)> {
        &self.params
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut cmdl = Argh::new();
    cmdl.parse(args[1..].to_vec());

    if cmdl.is_flag_set("-v") {
        println!("Verbose, I am.");
    }

    println!("Positional args:");
    for pos_arg in cmdl.pos_args() {
        println!("\t{}", pos_arg);
    }

    println!("Flags:");
    for flag in cmdl.flags() {
        println!("\t{}", flag);
    }

    println!("Parameters:");
    for param in cmdl.params() {
        println!("\t{} : {}", param.0, param.1);
    }

    exit(0);
}