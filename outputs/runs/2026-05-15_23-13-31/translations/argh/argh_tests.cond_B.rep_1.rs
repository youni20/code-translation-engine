use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::str::FromStr;

#[derive(Default)]
struct Parser {
    flags: HashSet<String>,
    params: HashMap<String, String>,
    pos_args: Vec<String>,
}

enum ParseMode {
    Default,
    PreferParamForUnregOption,
    NoSplitOnEqualsign,
    SingleDashIsMultiflag,
    PreferFlagForUnregOption,
}

impl Parser {
    fn parse(&mut self, argc: usize, argv: &[Option<&str>], mode: Option<ParseMode>) {
        // The implementation of parsing logic goes here based on the mode
        // Convert C++ logic to Rust
    }

    fn parse_args(&mut self, args: Vec<String>) {
        let argc = args.len();
        let argv: Vec<_> = args.iter().map(|s| Some(s.as_str())).collect();
        self.parse(argc, &argv, None);
    }

    fn add_param(&mut self, param: &str) {
        // Add logic to register parameter
    }

    fn add_params(&mut self, params: &[&str]) {
        for param in params {
            self.add_param(param);
        }
    }

    fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn get_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    fn get_param(&self, param: &str) -> Option<&str> {
        self.params.get(param).map(String::as_str)
    }

    fn get_positional_arg(&self, index: usize) -> Option<&str> {
        self.pos_args.get(index).map(String::as_str)
    }
}

fn main() {
    // Example usage of the Parser to simulate the test
    let mut cmdl = Parser::default();
    let argv = vec![
        Some("0"),
        Some("-a"),
        Some("1"),
        Some("-b"),
        Some("2"),
        Some("3"),
        Some("4"),
    ];
    let argc = argv.len();
    cmdl.parse(argc, &argv, None);

    assert_eq!(cmdl.pos_args().len(), 5);
    assert_eq!(cmdl.flags().len(), 2);
    assert_eq!(cmdl.params().len(), 0);
    assert!(cmdl.get_flag("a"));
    assert!(cmdl.get_flag("b"));
    assert!(!cmdl.get_flag("c"));
}