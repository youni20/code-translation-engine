use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[derive(Default)]
struct Parser {
    pos_args: Vec<String>,
    flags: HashSet<String>,
    params: HashMap<String, String>,
    options: HashMap<String, String>,  // to store additional parse options, if needed
}

impl Parser {
    fn new() -> Self {
        Self {
            pos_args: vec![],
            flags: HashSet::new(),
            params: HashMap::new(),
            options: HashMap::new(),
        }
    }

    fn parse(&mut self, argc: usize, argv: Option<&[&str]>, options: Option<&str>) {
        if let Some(args) = argv {
            for i in 0..argc {
                let arg = args[i];
                if arg.starts_with("--") {
                    let parts: Vec<&str> = arg[2..].split('=').collect();
                    if parts.len() == 2 {
                        self.params.insert(parts[0].to_string(), parts[1].to_string());
                    } else {
                        self.flags.insert(parts[0].to_string());
                    }
                } else if arg.starts_with('-') {
                    self.flags.insert(arg[1..].to_string());
                } else {
                    self.pos_args.push(arg.to_string());
                }
            }
        }
        if let Some(opt) = options {
            self.options.insert(opt.to_string(), String::new());
        }
    }

    fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    fn size(&self) -> usize {
        self.flags.len() + self.params.len()
    }

    fn check_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    fn check_param(&self, param: &str) -> Option<&String> {
        self.params.get(param)
    }

    fn get(&self, index: usize) -> Option<&String> {
        if index < self.pos_args.len() {
            Some(&self.pos_args[index])
        } else {
            None
        }
    }

    fn get_default<T: FromStr>(&self, index: usize, default: T) -> Result<T, T::Err> {
        if let Some(value) = self.get(index) {
            value.parse::<T>()
        } else {
            Ok(default)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_cmdl() {
        let mut cmdl = Parser::new();
        cmdl.parse(0, None, None);
        assert_eq!(cmdl.pos_args().len(), 0);
        assert_eq!(cmdl.size(), 0);
        assert!(cmdl.get(0).unwrap_or(&"".to_string()).is_empty());
        assert!(cmdl.get(10).unwrap_or(&"".to_string()).is_empty());
        assert!(!cmdl.check_flag("xxx"));
        assert!(cmdl.check_param("xxx").unwrap_or(&"".to_string()).is_empty());
    }

    #[test]
    fn test_parsing_ctor() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let argc = argv.len();
        let mut cmdl = Parser::new();
        cmdl.parse(argc, Some(&argv), None);
        assert_eq!(cmdl.flags().len(), 2);
        assert_eq!(cmdl.pos_args().len(), 5);
        assert_eq!(cmdl.size(), 5);
        assert!(cmdl.check_flag("a"));
        assert!(cmdl.check_flag("b"));
        assert!(!cmdl.check_flag("c"));
    }

    #[test]
    fn test_positional_access() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let argc = argv.len();
        let mut cmdl = Parser::new();
        cmdl.parse(argc, Some(&argv), None);
        assert_eq!(cmdl.pos_args().len(), 5);
        for parg in cmdl.pos_args() {
            assert!(!parg.is_empty());
        }
        assert_eq!(cmdl.get(0).unwrap(), "0");
        assert_eq!(cmdl.get(1).unwrap(), "1");
        assert_eq!(cmdl.get(2).unwrap(), "2");
        assert_eq!(cmdl.get(3).unwrap(), "3");
        assert_eq!(cmdl.get(4).unwrap(), "4");
        assert!(cmdl.get(10).unwrap_or(&"".to_string()).is_empty());

        assert!(cmdl.get(0).is_some());
        assert!(cmdl.get(1).is_some());
        assert!(cmdl.get(2).is_some());
        assert!(cmdl.get(3).is_some());
        assert!(cmdl.get(4).is_some());
        assert!(cmdl.get(10).is_none());

        let mut val = 0;
        assert!(cmdl.get_default(0, 7).is_ok());
        assert_eq!(cmdl.get_default::<i32>(0, 7).unwrap(), 0);
        assert!(cmdl.get_default(1, 7).is_ok());
        assert_eq!(cmdl.get_default::<i32>(1, 7).unwrap(), 1);
        assert!(cmdl.get_default(2, 7).is_ok());
        assert_eq!(cmdl.get_default::<i32>(2, 7).unwrap(), 2);
        assert!(cmdl.get_default(3, 7).is_ok());
        assert_eq!(cmdl.get_default::<i32>(3, 7).unwrap(), 3);
        assert!(cmdl.get_default(4, 7).is_ok());
        assert_eq!(cmdl.get_default::<i32>(4, 7).unwrap(), 4);
        assert!(cmdl.get_default::<i32>(5, val).is_err());
    }

    #[test]
    fn test_flag_access() {
        let argv = ["0", "-a", "1", "-b", "2", "3", "4"];
        let argc = argv.len();
        let mut cmdl = Parser::new();
        cmdl.parse(argc, Some(&argv), None);
        assert_eq!(cmdl.flags().len(), 2);
        assert_eq!(cmdl.pos_args().len(), 5);
        assert!(cmdl.check_flag("a"));
        assert!(cmdl.check_flag("b"));
        assert!(!cmdl.check_flag("c"));
    }

    #[test]
    fn test_parameter_access() {
        let argv = ["0", "-a", "-1", "-b", "2", "3", "4"];
        let argc = argv.len();
        let mut cmdl = Parser::new();
        cmdl.parse(argc, Some(&argv), Some("PREFER_PARAM_FOR_UNREG_OPTION"));
        assert_eq!(cmdl.params().len(), 2);
        assert_eq!(cmdl.pos_args().len(), 3);
        assert_eq!(cmdl.check_param("a").unwrap(), "-1");
        assert_eq!(cmdl.check_param("b").unwrap(), "2");
    }
    
    #[test]
    fn test_negative_numbers_are_not_options() {
        let argv = ["-1", "-0", "-0.4", "-1e6", "-1.3e-2"];
        let argc = argv.len();
        let mut cmdl = Parser::new();
        cmdl.parse(argc, Some(&argv), None);
        assert_eq!(cmdl.pos_args().len(), argc);
        assert_eq!(cmdl.params().len(), 0);
        assert_eq!(cmdl.flags().len(), 0);
    }

    // More tests can be added here following the structure and logic
    // of above Rust code translated from C++
}

fn main() {
    // For demonstration purposes. Actual tests are managed in #[cfg(test)].
}