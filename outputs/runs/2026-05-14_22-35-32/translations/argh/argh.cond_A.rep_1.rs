use std::collections::{HashMap, HashSet};

mod argh {
    use super::*;

    pub type StringStream = Option<String>;

    pub struct MultiMapIterationWrapper<'a> {
        map: &'a HashMap<String, Vec<String>>,
        key: String,
    }

    impl<'a> MultiMapIterationWrapper<'a> {
        pub fn new(map: &'a HashMap<String, Vec<String>>, key: String) -> Self {
            MultiMapIterationWrapper { map, key }
        }
    }

    impl<'a> IntoIterator for MultiMapIterationWrapper<'a> {
        type Item = &'a String;
        type IntoIter = std::slice::Iter<'a, String>;

        fn into_iter(self) -> Self::IntoIter {
            self.map.get(&self.key).map_or([].iter(), |v| v.iter())
        }
    }

    pub struct Parser {
        pos_args: Vec<String>,
        params: HashMap<String, Vec<String>>,
        flags: HashSet<String>,
        registered_params: HashSet<String>,
        args: Vec<String>,
    }

    impl Parser {
        pub const PREFER_FLAG_FOR_UNREG_OPTION: u8 = 1 << 0;
        pub const PREFER_PARAM_FOR_UNREG_OPTION: u8 = 1 << 1;
        pub const NO_SPLIT_ON_EQUALSIGN: u8 = 1 << 2;
        pub const SINGLE_DASH_IS_MULTIFLAG: u8 = 1 << 3;

        pub fn new() -> Self {
            Self {
                pos_args: Vec::new(),
                params: HashMap::new(),
                flags: HashSet::new(),
                registered_params: HashSet::new(),
                args: Vec::new(),
            }
        }

        pub fn with_pre_reg_names<I>(pre_reg_names: I) -> Self
        where
            I: IntoIterator<Item = &'static str>,
        {
            let mut parser = Self::new();
            parser.add_params(pre_reg_names);
            parser
        }

        pub fn from_argv(argv: &[&str], mode: u8) -> Self {
            let mut parser = Self::new();
            parser.parse(argv, mode);
            parser
        }

        pub fn parse(&mut self, argv: &[&str], mode: u8) {
            self.flags.clear();
            self.params.clear();
            self.pos_args.clear();
            self.args = argv.iter().map(|&arg| arg.to_string()).collect();

            let mut i = 0;
            while i < self.args.len() {
                if !self.is_option(&self.args[i]) {
                    self.pos_args.push(self.args[i].clone());
                    i += 1;
                    continue;
                }

                let mut name = self.trim_leading_dashes(&self.args[i]);

                if mode & Self::NO_SPLIT_ON_EQUALSIGN == 0 {
                    if let Some(equal_pos) = name.find('=') {
                        self.params.entry(name[..equal_pos].to_string())
                            .or_default()
                            .push(name[equal_pos + 1..].to_string());
                        i += 1;
                        continue;
                    }
                }

                if self.args[i].starts_with('-') 
                    && mode & Self::SINGLE_DASH_IS_MULTIFLAG > 0 
                    && name.len() >= 2 
                    && !self.is_param(&name)
                {
                    let mut keep_param = String::new();
                    if let Some(last) = name.chars().last() {
                        if self.is_param(&last.to_string()) {
                            keep_param = last.to_string();
                            name.pop();
                        }
                    }

                    for c in name.chars() {
                        self.flags.insert(c.to_string());
                    }

                    if !keep_param.is_empty() {
                        name = keep_param;
                    } else {
                        i += 1;
                        continue;
                    }
                }

                if i == self.args.len() - 1 || self.is_option(&self.args[i + 1]) {
                    self.flags.insert(name);
                    i += 1;
                    continue;
                }

                let prefer_param = mode & Self::PREFER_PARAM_FOR_UNREG_OPTION > 0;
                assert_ne!(
                    mode & Self::PREFER_FLAG_FOR_UNREG_OPTION > 0 && mode & Self::PREFER_PARAM_FOR_UNREG_OPTION > 0,
                    true
                );

                if self.is_param(&name) || prefer_param {
                    self.params.entry(name)
                        .or_default()
                        .push(self.args[i + 1].clone());
                    i += 2;
                } else {
                    self.flags.insert(name);
                    i += 1;
                }
            }
        }

        pub fn add_param(&mut self, name: &str) {
            self.registered_params.insert(self.trim_leading_dashes(name));
        }

        pub fn add_params<I>(&mut self, init_list: I)
        where
            I: IntoIterator<Item = &'static str>,
        {
            for name in init_list.into_iter() {
                self.add_param(name);
            }
        }

        pub fn flags(&self) -> &HashSet<String> {
            &self.flags
        }

        pub fn params(&self) -> &HashMap<String, Vec<String>> {
            &self.params
        }

        pub fn params_for(&self, name: &str) -> MultiMapIterationWrapper {
            let trimmed_name = self.trim_leading_dashes(name).to_string();
            MultiMapIterationWrapper::new(&self.params, trimmed_name)
        }

        pub fn pos_args(&self) -> &Vec<String> {
            &self.pos_args
        }

        pub fn is_option(&self, arg: &str) -> bool {
            !self.is_number(arg) && arg.starts_with('-')
        }

        pub fn is_number(&self, arg: &str) -> bool {
            arg.parse::<f64>().is_ok()
        }

        pub fn trim_leading_dashes(&self, name: &str) -> String {
            name.trim_start_matches('-').to_string()
        }

        pub fn got_flag(&self, name: &str) -> bool {
            let trimmed = self.trim_leading_dashes(name);
            self.flags.contains(&trimmed)
        }

        pub fn is_param(&self, name: &str) -> bool {
            self.registered_params.contains(name)
        }

        pub fn get_flag(&self, name: &str) -> bool {
            self.got_flag(name)
        }

        pub fn get_flags<I>(&self, init_list: I) -> bool
        where
            I: IntoIterator<Item = &'static str>,
        {
            init_list.into_iter().any(|name| self.get_flag(name))
        }

        pub fn get_positional(&self, ind: usize) -> Option<&str> {
            self.pos_args.get(ind).map(|s| s.as_str())
        }

        pub fn get_param_stream(&self, name: &str) -> StringStream {
            let trimmed_name = self.trim_leading_dashes(name);
            self.params.get(&trimmed_name)
                .and_then(|args| args.last().cloned())
                .map(Some)
                .unwrap_or_else(|| None)
        }

        pub fn get_multi_param_stream<I>(&self, init_list: I) -> StringStream
        where
            I: IntoIterator<Item = &'static str>,
        {
            for name in init_list {
                if let Some(param) = self.get_param_stream(name) {
                    return Some(param);
                }
            }
            None
        }
    }
}

fn main() {
    let mut parser = argh::Parser::from_argv(&["program", "-f", "filename", "positional"], 0);
    parser.parse(&["program", "-f", "filename", "positional"], argh::Parser::PREFER_PARAM_FOR_UNREG_OPTION);
}