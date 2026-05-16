struct Parser {
    flags: std::collections::HashSet<String>,
    params: std::collections::HashMap<String, String>,
    pos_args: Vec<String>,
}

impl Parser {
    fn new() -> Self {
        Self {
            flags: std::collections::HashSet::new(),
            params: std::collections::HashMap::new(),
            pos_args: Vec::new(),
        }
    }

    fn parse(&mut self, args: &[&str]) {
        let mut i = 0;
        while i < args.len() {
            if args[i].starts_with('-') {
                let flag = args[i].trim_start_matches('-').to_string();
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    self.params.insert(flag.clone(), args[i + 1].to_string());
                } else {
                    self.flags.insert(flag);
                }
            } else {
                self.pos_args.push(args[i].to_string());
            }
            i += 1;
        }
    }

    fn flags(&self) -> &std::collections::HashSet<String> {
        &self.flags
    }

    fn params(&self) -> &std::collections::HashMap<String, String> {
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

    fn get_pos_arg(&self, index: usize) -> Option<&String> {
        self.pos_args.get(index)
    }

    fn size(&self) -> usize {
        self.flags.len() + self.params.len() + self.pos_args.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cmdl() {
        let mut parser = Parser::new();
        parser.parse(&[]);
        assert_eq!(parser.pos_args().len(), 0);
        assert_eq!(parser.size(), 0);
        assert!(parser.get_pos_arg(0).is_none());
        assert_eq!(parser.get_param("xxx").is_none(), true);
    }

    #[test]
    fn test_parsing_ctor() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let mut parser = Parser::new();
        parser.parse(args);
        assert_eq!(parser.flags().len(), 2);
        assert_eq!(parser.pos_args().len(), 5);
        assert!(parser.get_flag("a"));
        assert!(parser.get_flag("b"));
        assert!(!parser.get_flag("c"));
    }

    #[test]
    fn test_positional_access() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let mut parser = Parser::new();
        parser.parse(args);
        assert_eq!(parser.pos_args().len(), 5);

        for parg in parser.pos_args() {
            assert!(!parg.is_empty());
        }

        assert_eq!(parser.get_pos_arg(0).unwrap(), "0");
        assert_eq!(parser.get_pos_arg(1).unwrap(), "1");
        assert_eq!(parser.get_pos_arg(2).unwrap(), "2");
        assert_eq!(parser.get_pos_arg(3).unwrap(), "3");
        assert_eq!(parser.get_pos_arg(4).unwrap(), "4");

        assert!(parser.get_pos_arg(5).is_none());
    }

    #[test]
    fn test_flag_access() {
        let args = &["0", "-a", "1", "-b", "2", "3", "4"];
        let mut parser = Parser::new();
        parser.parse(args);
        assert!(parser.get_flag("a"));
        assert!(parser.get_flag("b"));
        assert!(!parser.get_flag("c"));
    }
}

fn main() {
    // Example usage of Parser
    let args = &["0", "-a", "1", "-b", "2", "3", "4"];
    let mut parser = Parser::new();
    parser.parse(args);

    println!("Flags: {:?}", parser.flags());
    println!("Params: {:?}", parser.params());
    println!("Positional Args: {:?}", parser.pos_args());
}