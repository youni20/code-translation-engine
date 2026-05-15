use std::env;
use std::process::ExitCode;

struct Parser {
    positional_args: Vec<String>,
    flags: Vec<String>,
    params: Vec<(String, String)>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            positional_args: Vec::new(),
            flags: Vec::new(),
            params: Vec::new(),
        }
    }

    fn parse(&mut self, args: Vec<String>) {
        let mut iter = args.into_iter().peekable();
        
        while let Some(arg) = iter.next() {
            if arg.starts_with('-') {
                if let Some(next_arg) = iter.peek() {
                    if !next_arg.starts_with('-') {
                        self.params.push((arg, iter.next().unwrap()));
                    } else {
                        self.flags.push(arg);
                    }
                } else {
                    self.flags.push(arg);
                }
            } else {
                self.positional_args.push(arg);
            }
        }
    }

    fn is_flag_present(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut cmdl = Parser::new();
    cmdl.parse(args);

    if cmdl.is_flag_present("-v") {
        println!("Verbose, I am.");
    }

    println!("Positional args:");
    for pos_arg in &cmdl.positional_args {
        println!("\t{}", pos_arg);
    }

    println!("\nFlags:");
    for flag in &cmdl.flags {
        println!("\t{}", flag);
    }

    println!("\nParameters:");
    for param in &cmdl.params {
        println!("\t{} : {}", param.0, param.1);
    }

    ExitCode::SUCCESS
}