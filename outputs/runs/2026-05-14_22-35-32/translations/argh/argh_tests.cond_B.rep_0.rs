use std::collections::{HashMap, HashSet};
use std::env;

#[derive(Default)]
struct Parser {
    flags: HashSet<String>,
    params: HashMap<String, String>,
    pos_args: Vec<String>,
}

impl Parser {
    fn parse(&mut self, args: &[&str]) {
        let mut args_iter = args.iter().peekable();
        while let Some(arg) = args_iter.next() {
            if arg.starts_with('-') {
                let arg = arg.trim_start_matches('-');
                if args_iter.peek().is_some() && !args_iter.peek().unwrap().starts_with('-') {
                    self.params.insert(arg.to_string(), args_iter.next().unwrap().to_string());
                } else {
                    self.flags.insert(arg.to_string());
                }
            } else {
                self.pos_args.push(arg.to_string());
            }
        }
    }

    fn add_param(&mut self, param: &str) {
        // By default, this is a no-op.
    }

    fn add_params(&mut self, params: &[&str]) {
        // By default, this is a no-op.
    }

    fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    fn params(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn size(&self) -> usize {
        self.pos_args.len()
    }

    fn new() -> Self {
        Self::default()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    
    let mut parser = Parser::new();
    parser.parse(&args_ref);
    
    // Example checks based on the C++ implementation:
    assert_eq!(parser.size(), parser.pos_args().len());
    assert_eq!(parser.flags().is_empty(), parser.pos_args().is_empty());
    assert!(parser.flags().is_empty());
    assert!(parser.pos_args().is_empty());
}