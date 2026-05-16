use std::env;

mod argh {
    use std::collections::{HashMap, HashSet};
    use std::iter;

    pub struct Parser {
        positional_args: Vec<String>,
        flags: HashSet<String>,
        params: HashMap<String, String>,
    }

    impl Parser {
        pub const PREFER_PARAM_FOR_UNREG_OPTION: u8 = 0;

        pub fn new() -> Self {
            Self {
                positional_args: Vec::new(),
                flags: HashSet::new(),
                params: HashMap::new(),
            }
        }

        pub fn parse(&mut self, args: Vec<String>, _preference: u8) {
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                if arg.starts_with('-') {
                    if let Some(value) = iter.next() {
                        if value.starts_with('-') {
                            self.flags.insert(arg);
                            iter = iter::once(value).chain(iter).collect::<Vec<_>>().into_iter().skip(0);
                        } else {
                            self.params.insert(arg, value);
                        }
                    } else {
                        self.flags.insert(arg);
                    }
                } else {
                    self.positional_args.push(arg);
                }
            }
        }

        pub fn pos_args(&self) -> impl Iterator<Item = &String> + '_ {
            self.positional_args.iter()
        }

        pub fn flags(&self) -> impl Iterator<Item = &String> + '_ {
            self.flags.iter()
        }

        pub fn params(&self) -> impl Iterator<Item = (&String, &String)> + '_ {
            self.params.iter()
        }

        pub fn get(&self, key: &str) -> Option<&String> {
            self.params.get(key)
        }

        pub fn contains_key(&self, key: &str) -> bool {
            self.flags.contains(key)
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cmdl = argh::Parser::new();
    cmdl.parse(args, argh::Parser::PREFER_PARAM_FOR_UNREG_OPTION);

    if cmdl.contains_key("-v") {
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
}