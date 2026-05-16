mod argh {
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    pub struct StringStream(String);

    impl StringStream {
        pub fn new(value: &str) -> Self {
            StringStream(value.to_string())
        }

        pub fn failed(&self) -> bool {
            false
        }

        pub fn str(&self) -> &str {
            &self.0
        }
    }

    pub struct MultimapIterationWrapper<'a> {
        lb: std::collections::btree_map::Range<'a, String, String>,
    }

    impl<'a> MultimapIterationWrapper<'a> {
        pub fn new(
            lb: std::collections::btree_map::Range<'a, String, String>,
        ) -> Self {
            MultimapIterationWrapper { lb }
        }
    }

    pub struct Parser {
        args: Vec<String>,
        params: BTreeMap<String, String>,
        pos_args: Vec<String>,
        flags: BTreeSet<String>,
        registered_params: HashSet<String>,
        empty: String,
    }

    impl Default for Parser {
        fn default() -> Self {
            Self {
                args: Vec::new(),
                params: BTreeMap::new(),
                pos_args: Vec::new(),
                flags: BTreeSet::new(),
                registered_params: HashSet::new(),
                empty: String::new(),
            }
        }
    }

    impl Parser {
        pub const PREFER_FLAG_FOR_UNREG_OPTION: i32 = 1 << 0;
        pub const PREFER_PARAM_FOR_UNREG_OPTION: i32 = 1 << 1;
        pub const NO_SPLIT_ON_EQUALSIGN: i32 = 1 << 2;
        pub const SINGLE_DASH_IS_MULTIFLAG: i32 = 1 << 3;

        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_param(&mut self, name: &str) {
            self.registered_params
                .insert(Self::trim_leading_dashes(name));
        }

        pub fn add_params(&mut self, name: &str) {
            self.add_param(name);
        }

        pub fn add_param_list(&mut self, init_list: &[&str]) {
            self.add_params_list(init_list);
        }

        pub fn add_params_list(&mut self, init_list: &[&str]) {
            for name in init_list {
                self.registered_params
                    .insert(Self::trim_leading_dashes(name));
            }
        }

        pub fn parse(&mut self, argv: &[&str], mode: i32) {
            self.flags.clear();
            self.params.clear();
            self.pos_args.clear();

            self.args = argv.iter().map(|&arg| arg.to_string()).collect();

            let mut i = 0;
            while i < self.args.len() {
                let arg = &self.args[i];
                if !self.is_option(arg) {
                    self.pos_args.push(arg.clone());
                    i += 1;
                    continue;
                }

                let mut name = Self::trim_leading_dashes(arg);

                if mode & Self::NO_SPLIT_ON_EQUALSIGN == 0 {
                    if let Some(equal_pos) = name.find('=') {
                        self.params.insert(
                            name[..equal_pos].to_string(),
                            name[equal_pos + 1..].to_string(),
                        );
                        i += 1;
                        continue;
                    }
                }

                if arg.len() - name.len() == 1
                    && mode & Self::SINGLE_DASH_IS_MULTIFLAG > 0
                    && !self.is_param(&name)
                {
                    let mut keep_param: Option<char> = None;

                    if let Some(c) = name.chars().last() {
                        if self.is_param(&c.to_string()) {
                            keep_param = Some(c);
                            name.pop();
                        }
                    }

                    for c in name.chars() {
                        self.flags.insert(c.to_string());
                    }

                    if let Some(c) = keep_param {
                        name = c.to_string();
                    } else {
                        i += 1;
                        continue;
                    }
                }

                if i == self.args.len() - 1 || self.is_option(&self.args[i + 1]) {
                    self.flags.insert(name.clone());
                    i += 1;
                    continue;
                }

                let prefer_param =
                    mode & Self::PREFER_PARAM_FOR_UNREG_OPTION > 0;

                if self.is_param(&name) || prefer_param {
                    self.params.insert(name, self.args[i + 1].clone());
                    i += 2;
                } else {
                    self.flags.insert(name);
                    i += 1;
                }
            }
        }

        pub fn flags(&self) -> &BTreeSet<String> {
            &self.flags
        }

        pub fn params(&self) -> &BTreeMap<String, String> {
            &self.params
        }

        pub fn pos_args(&self) -> &Vec<String> {
            &self.pos_args
        }

        pub fn size(&self) -> usize {
            self.pos_args.len()
        }

        pub fn got_flag(&self, name: &str) -> bool {
            self.flags
                .contains(&Self::trim_leading_dashes(name).to_string())
        }

        pub fn is_param(&self, name: &str) -> bool {
            self.registered_params.contains(name)
        }

        pub fn is_option(&self, arg: &str) -> bool {
            if Parser::is_number(arg) {
                return false;
            }
            arg.starts_with('-')
        }

        fn trim_leading_dashes(name: &str) -> String {
            name.trim_start_matches('-').to_string()
        }

        fn is_number(arg: &str) -> bool {
            arg.parse::<f64>().is_ok()
        }

        pub fn bad_stream(&self) -> StringStream {
            StringStream::new("")
        }
    }

    impl std::ops::Index<&str> for Parser {
        type Output = String;

        fn index(&self, index: &str) -> &Self::Output {
            if self.got_flag(index) {
                return &self.empty;
            }
            &self.empty
        }
    }

    impl std::ops::Index<usize> for Parser {
        type Output = String;

        fn index(&self, index: usize) -> &Self::Output {
            if index < self.pos_args.len() {
                &self.pos_args[index]
            } else {
                &self.empty
            }
        }
    }

    impl std::ops::Index<std::ops::Range<usize>> for Parser {
        type Output = [String];

        fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
            &self.pos_args[index]
        }
    }
}

fn main() {
    // Entry point to satisfy the compiler, though no executable functionality is specified.
}