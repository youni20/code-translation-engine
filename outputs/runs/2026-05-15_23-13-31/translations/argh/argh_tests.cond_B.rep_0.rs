use std::collections::{HashSet, HashMap};
use std::vec::Vec;

struct Parser {
    flags: HashSet<String>,
    params: HashMap<String, String>,
    pos_args: Vec<String>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            flags: HashSet::new(),
            params: HashMap::new(),
            pos_args: Vec::new(),
        }
    }

    fn parse(&mut self, argc: usize, argv: Option<&[&str]>) {
        if let Some(args) = argv {
            let mut it = args.iter().take(argc).peekable();
            while let Some(&arg) = it.next() {
                if arg.starts_with("--") {
                    let parts: Vec<&str> = arg[2..].split('=').collect();
                    let key = parts[0].to_string();
                    if parts.len() > 1 {
                        self.params.insert(key, parts[1].to_string());
                    } else {
                        self.flags.insert(key);
                    }
                } else if arg.starts_with('-') {
                    let key = &arg[1..];
                    if let Some(&next_arg) = it.peek() {
                        if !next_arg.starts_with('-') {
                            self.params.insert(key.to_string(), it.next().unwrap().to_string());
                        } else {
                            self.flags.insert(key.to_string());
                        }
                    } else {
                        self.flags.insert(key.to_string());
                    }
                } else {
                    self.pos_args.push(arg.to_string());
                }
            }
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

    fn get_param(&self, param: &str) -> Option<&String> {
        self.params.get(param)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_parser_with_args(args: &[&str]) -> Parser {
        let mut parser = Parser::new();
        parser.parse(args.len(), Some(args));
        parser
    }

    #[test]
    fn test_empty_cmdl() {
        let mut cmdl = Parser::new();
        cmdl.parse(0, None);
        assert_eq!(0, cmdl.pos_args().len());
        assert_eq!(0, cmdl.flags().len());
        assert_eq!(0, cmdl.params().len());
    }

    #[test]
    fn test_parsing_ctor() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let cmdl = setup_parser_with_args(args);
        assert_eq!(2, cmdl.flags.len());
        assert_eq!(5, cmdl.pos_args.len());
        assert!(cmdl.get_flag("a"));
        assert!(cmdl.get_flag("b"));
        assert!(!cmdl.get_flag("c"));
    }

    #[test]
    fn test_positional_access() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let cmdl = setup_parser_with_args(args);
        assert_eq!(5, cmdl.pos_args.len());
        for i in 0..cmdl.pos_args.len() {
            assert!(!cmdl.pos_args[i].is_empty());
        }
        assert_eq!(cmdl.pos_args[0], "0");
        assert_eq!(cmdl.pos_args[1], "1");
        assert_eq!(cmdl.pos_args[2], "2");
        assert_eq!(cmdl.pos_args[3], "3");
        assert_eq!(cmdl.pos_args[4], "4");
        assert!(cmdl.get_param("non-existing").is_none());
    }

    #[test]
    fn test_flag_access() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let cmdl = setup_parser_with_args(args);
        assert_eq!(2, cmdl.flags.len());
        assert!(cmdl.get_flag("a"));
        assert!(cmdl.get_flag("b"));
        assert!(!cmdl.get_flag("c"));
    }

    #[test]
    fn test_parameter_access() {
        let args = &["0", "-a", "-1", "-b", "2", "3", "4"];
        let cmdl = setup_parser_with_args(args);
        assert_eq!(2, cmdl.params.len());
        assert_eq!(cmdl.get_param("a"), Some(&"-1".to_string()));
        assert_eq!(cmdl.get_param("b"), Some(&"2".to_string()));
    }

    #[test]
    fn test_negative_numbers_not_options() {
        let args = &["-1", "-0", "-0.4", "-1e6", "-1.3e-2"];
        let cmdl = setup_parser_with_args(args);
        assert_eq!(args.len(), cmdl.pos_args.len());
        assert_eq!(0, cmdl.params.len());
        assert_eq!(0, cmdl.flags.len());
    }

    #[test]
    fn test_handles_const_char_versions() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let cmdl = setup_parser_with_args(args);
        assert_eq!(5, cmdl.pos_args.len());
        assert_eq!(2, cmdl.flags.len());
    }
}

fn main() {
    // Example usage of Parser
    let args = &["-a", "--option=value", "positional", "-b"];
    let mut parser = Parser::new();
    parser.parse(args.len(), Some(args));

    println!("Flags: {:?}", parser.flags());
    println!("Params: {:?}", parser.params());
    println!("Positional Arguments: {:?}", parser.pos_args());
}