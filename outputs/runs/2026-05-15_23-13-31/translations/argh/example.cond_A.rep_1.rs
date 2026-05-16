use std::env;
use std::process::exit;
use std::collections::HashMap;

struct Parser {
    // simulate the internal argh state
    flags: Vec<String>,
    params: HashMap<String, String>,
    pos_args: Vec<String>,
}

impl Parser {
    fn new() -> Self {
        Self {
            flags: Vec::new(),
            params: HashMap::new(),
            pos_args: Vec::new(),
        }
    }

    fn parse(&mut self, args: Vec<String>) {
        let mut iter = args.into_iter();
        iter.next(); // skip the executable name
        while let Some(arg) = iter.next() {
            if arg.starts_with('-') {
                if let Some(next_arg) = iter.next() {
                    if !next_arg.starts_with('-') {
                        self.params.insert(arg, next_arg);
                    } else {
                        self.flags.push(arg);
                        self.pos_args.push(next_arg);
                    }
                } else {
                    self.flags.push(arg);
                }
            } else {
                self.pos_args.push(arg);
            }
        }
    }

    fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }

    fn pos_args(&self) -> &[String] {
        &self.pos_args
    }

    fn flags(&self) -> impl Iterator<Item = &String> {
        self.flags.iter()
    }

    fn params(&self) -> impl Iterator<Item = (&String, &String)> {
        self.params.iter()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdl = Parser::new();
    cmdl.parse(args);

    if cmdl.has_flag("-v") {
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