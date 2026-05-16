use std::env;
use std::process::exit;

struct Parser {
    args: Vec<String>,
    pos_args: Vec<String>,
    flags: Vec<String>,
    params: std::collections::HashMap<String, String>,
}

impl Parser {
    fn new() -> Self {
        Self {
            args: Vec::new(),
            pos_args: Vec::new(),
            flags: Vec::new(),
            params: std::collections::HashMap::new(),
        }
    }

    fn parse(&mut self, args: Vec<String>) {
        self.args = args;
        let mut iter = self.args.iter().skip(1).peekable();
        while let Some(arg) = iter.next() {
            if arg.starts_with('-') {
                if let Some(next_arg) = iter.peek() {
                    if next_arg.starts_with('-') {
                        self.flags.push(arg.clone());
                    } else {
                        self.params.insert(arg.clone(), next_arg.to_string());
                        iter.next();
                    }
                } else {
                    self.flags.push(arg.clone());
                }
            } else {
                self.pos_args.push(arg.clone());
            }
        }
    }

    fn is_flag_set(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string())
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    fn params(&self) -> &std::collections::HashMap<String, String> {
        &self.params
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdl = Parser::new();
    cmdl.parse(args);

    if cmdl.is_flag_set("-v") {
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