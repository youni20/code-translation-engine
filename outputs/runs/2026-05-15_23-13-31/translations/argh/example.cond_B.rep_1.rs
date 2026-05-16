use std::env;

struct Parser {
    args: Vec<String>,
    flags: Vec<String>,
    params: Vec<(String, String)>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            args: Vec::new(),
            flags: Vec::new(),
            params: Vec::new(),
        }
    }

    fn parse(&mut self, args: Vec<String>) {
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg.starts_with('-') {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    self.params.push((arg.clone(), args[i + 1].clone()));
                    i += 1;
                } else {
                    self.flags.push(arg.clone());
                }
            } else {
                self.args.push(arg.clone());
            }
            i += 1;
        }
    }

    fn is_flag_present(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }

    fn positional_args(&self) -> &Vec<String> {
        &self.args
    }

    fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    fn params(&self) -> &Vec<(String, String)> {
        &self.params
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut cmdl = Parser::new();
    cmdl.parse(args);

    if cmdl.is_flag_present("-v") {
        println!("Verbose, I am.");
    }

    println!("Positional args:");
    for pos_arg in cmdl.positional_args() {
        println!("\t{}", pos_arg);
    }

    println!("\nFlags:");
    for flag in cmdl.flags() {
        println!("\t{}", flag);
    }

    println!("\nParameters:");
    for param in cmdl.params() {
        println!("\t{} : {}", param.0, param.1);
    }
}