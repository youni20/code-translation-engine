use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Parser {
    flags: HashSet<String>,
    params: HashMap<String, String>,
    pos_args: Vec<String>,
}

impl Parser {
    fn new() -> Self {
        Parser::default()
    }

    fn parse(&mut self, args: &[&str], prefer_flag: bool, single_dash_multiflag: bool) {
        let mut args_iter = args.iter().peekable();
        while let Some(&arg) = args_iter.next() {
            if arg.starts_with("--") {
                let equals_index = arg.find('=');
                if let Some(index) = equals_index {
                    let key = arg[2..index].to_string();
                    let value = arg[index + 1..].to_string();
                    self.params.insert(key, value);
                } else {
                    let stripped_key = &arg[2..];
                    if prefer_flag {
                        self.flags.insert(stripped_key.to_string());
                    } else if args_iter.peek().map_or(false, |&next| !next.starts_with("-")) {
                        if let Some(value) = args_iter.next() {
                            self.params.insert(stripped_key.to_string(), value.to_string());
                        }
                    }
                }
            } else if arg.starts_with('-') {
                if single_dash_multiflag {
                    for ch in arg[1..].chars() {
                        self.flags.insert(ch.to_string());
                    }
                } else {
                    let stripped_key = &arg[1..];
                    if prefer_flag {
                        self.flags.insert(stripped_key.to_string());
                    } else if args_iter.peek().map_or(false, |&next| !next.starts_with("-")) {
                        if let Some(value) = args_iter.next() {
                            self.params.insert(stripped_key.to_string(), value.to_string());
                        }
                    }
                }
            } else {
                self.pos_args.push(arg.to_string());
            }
        }
    }

    fn pos_args(&self) -> &[String] {
        &self.pos_args
    }

    fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    fn params(&self) -> &HashMap<String, String> {
        &self.params
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

    #[test]
    fn test_empty_cmdl() {
        let mut cmdl = Parser::new();
        cmdl.parse(&[], false, false);
        assert_eq!(0, cmdl.pos_args().len());
        assert_eq!(0, cmdl.flags().len());
        assert_eq!(0, cmdl.params().len());
        assert!(!cmdl.get_flag("xxx"));
        assert!(cmdl.get_param("xxx").is_none());
    }

    #[test]
    fn test_parsing_ctor() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        {
            let mut cmdl = Parser::new();
            cmdl.parse(&argv, true, false);
            assert_eq!(2, cmdl.flags().len());
            assert_eq!(5, cmdl.pos_args().len());
            assert!(cmdl.get_flag("a"));
            assert!(cmdl.get_flag("b"));
            assert!(!cmdl.get_flag("c"));
        }
    }

    #[test]
    fn test_positional_access() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv, true, false);
        assert_eq!(5, cmdl.pos_args().len());
        for parg in cmdl.pos_args() {
            assert!(!parg.is_empty());
        }
        assert_eq!(cmdl.get_param("0").unwrap(), "0");
        assert_eq!(cmdl.get_param("1").unwrap(), "1");
        assert_eq!(cmdl.get_param("2").unwrap(), "2");
        assert_eq!(cmdl.get_param("3").unwrap(), "3");
        assert_eq!(cmdl.get_param("4").unwrap(), "4");
        assert!(cmdl.get_param("argc+10").is_none());
    }

    #[test]
    fn test_flag_access() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv, true, false);
        assert_eq!(2, cmdl.flags().len());
        assert!(cmdl.get_flag("a"));
        assert!(cmdl.get_flag("b"));
    }

    #[test]
    fn test_parameter_access() {
        let argv = ["0", "-a", "-1", "-b", "2", "3", "4"];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv, false, false);
        assert_eq!(2, cmdl.params().len());
        assert_eq!("2", cmdl.get_param("b").unwrap());
    }

    #[test]
    fn test_split_parameter_on_equal() {
        let argv = ["--answer=42", "---no_val="];
        let mut cmdl = Parser::new();
        cmdl.parse(&argv, false, false);
        assert_eq!("42", cmdl.get_param("answer").unwrap());
        assert_eq!("", cmdl.get_param("no_val").unwrap());
    }
}

fn main() {}