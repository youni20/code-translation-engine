use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Parser {
    flags: HashSet<String>,
    pos_args: Vec<String>,
    params: HashMap<String, String>,
}

impl Parser {
    fn parse<'a>(&mut self, args: impl IntoIterator<Item = &'a str>) {
        let mut iter = args.into_iter().peekable();
        while let Some(arg) = iter.next() {
            if arg.starts_with("--") {
                let mut split = arg[2..].splitn(2, '=');
                let name = split.next().unwrap().to_string();
                let value = split.next().unwrap_or_default().to_string();
                self.params.insert(name, value);
            } else if arg.starts_with('-') {
                let flag = arg[1..].to_string();
                self.flags.insert(flag);
            } else {
                self.pos_args.push(arg.to_string());
            }
        }
    }

    fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    fn get_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    fn get_param(&self, param: &str) -> Option<&str> {
        self.params.get(param).map(|s| s.as_str())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut parser = Parser::default();
    parser.parse(args.iter().map(String::as_str).skip(1));

    // Example usage:
    if parser.get_flag("example") {
        println!("Example flag is set");
    }

    if let Some(value) = parser.get_param("key") {
        println!("Key is set to: {}", value);
    }
}