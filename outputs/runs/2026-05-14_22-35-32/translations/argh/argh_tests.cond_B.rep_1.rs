use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Default, Debug)]
struct Parser {
    flags: HashSet<String>,
    pos_args: Vec<String>,
    params: HashMap<String, Vec<String>>,
}

impl Parser {
    fn new() -> Self {
        Default::default()
    }

    fn parse(&mut self, args: &[&str]) {
        let mut iter = args.iter().peekable();
        while let Some(&arg) = iter.next() {
            if arg.starts_with('-') {
                let stripped = arg.trim_start_matches('-');
                if let Some(next_arg) = iter.peek() {
                    if !next_arg.starts_with('-') {
                        if !self.params.contains_key(stripped) {
                            self.params.insert(stripped.to_string(), vec![]);
                        }
                        self.params.get_mut(stripped).unwrap().push(next_arg.to_string());
                        iter.next();
                    } else {
                        self.flags.insert(stripped.to_string());
                    }
                } else {
                    self.flags.insert(stripped.to_string());
                }
            } else {
                self.pos_args.push(arg.to_string());
            }
        }
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    fn params(&self) -> &HashMap<String, Vec<String>> {
        &self.params
    }

    fn is_empty(&self) -> bool {
        self.flags.is_empty() && self.pos_args.is_empty() && self.params.is_empty()
    }

    fn get_arg(&self, key: &str) -> Option<&Vec<String>> {
        self.params.get(key)
    }

    fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    fn add_flag(&mut self, flag: &str) {
        self.flags.insert(flag.to_string());
    }

    fn add_param(&mut self, key: &str, value: &str) {
        self.params
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cmdl() {
        let mut cmdl = Parser::new();
        cmdl.parse(&[]);
        assert_eq!(0, cmdl.pos_args().len());
        assert!(cmdl.is_empty());
    }

    #[test]
    fn test_parsing_ctor() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv);
        assert_eq!(2, cmdl.flags().len());
        assert_eq!(5, cmdl.pos_args().len());
    }

    #[test]
    fn test_positional_access() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv);
        assert_eq!(5, cmdl.pos_args().len());
        for parg in cmdl.pos_args().iter() {
            assert!(!parg.is_empty());
        }

        assert_eq!("0", cmdl.pos_args()[0]);
        assert_eq!("1", cmdl.pos_args()[1]);
        assert_eq!("2", cmdl.pos_args()[2]);
        assert_eq!("3", cmdl.pos_args()[3]);
        assert_eq!("4", cmdl.pos_args()[4]);
    }

    #[test]
    fn test_flag_access() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv);
        assert!(cmdl.has_flag("a"));
        assert!(cmdl.has_flag("b"));
        assert!(!cmdl.has_flag("c"));
    }

    #[test]
    fn test_parameter_access() {
        let argv = ["0", "-a", "-1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv);
        assert_eq!(Some(&vec!["-1".to_string()]), cmdl.get_arg("a"));
        assert_eq!(Some(&vec!["2".to_string()]), cmdl.get_arg("b"));
    }
}

fn main() {
    // Main function for demonstration purposes.
}