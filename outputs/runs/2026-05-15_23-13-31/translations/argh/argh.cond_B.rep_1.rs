use std::collections::{BTreeMap, HashSet};

type StringStream = std::string::String;

// Until GCC 5, istringstream did not have a move constructor in C++, 
// so this workaround is equivalent. In Rust, we don't need that workaround.
fn make_string_stream<T: ToString>(value: T) -> String {
    value.to_string()
}

pub struct MultimapIterationWrapper<'a, K, V> {
    range: std::collections::btree_map::Range<'a, K, V>,
}

impl<'a, K, V> MultimapIterationWrapper<'a, K, V> {
    pub fn new(map: &'a BTreeMap<K, V>, key: &K) -> Self 
    where
        K: Ord,
    {
        let range = map.range(key..);
        MultimapIterationWrapper { range }
    }

    pub fn begin(&'a mut self) -> Option<(&'a K, &'a V)> {
        self.range.next()
    }

    pub fn end(&self) -> Option<&(&'a K, &'a V)> {
        None
    }

    pub fn size(&self) -> usize {
        self.range.clone().count()
    }
}

pub struct Parser {
    args: Vec<String>,
    params: BTreeMap<String, String>,
    pos_args: Vec<String>,
    flags: HashSet<String>,
    registered_params: HashSet<String>,
    empty: String,
}

impl Parser {
    pub const PREFER_FLAG_FOR_UNREG_OPTION: i32 = 1 << 0;
    pub const PREFER_PARAM_FOR_UNREG_OPTION: i32 = 1 << 1;
    pub const NO_SPLIT_ON_EQUALSIGN: i32 = 1 << 2;
    pub const SINGLE_DASH_IS_MULTIFLAG: i32 = 1 << 3;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &[&str]) -> Self {
        let mut parser = Self::new();
        parser.parse(args, Self::PREFER_FLAG_FOR_UNREG_OPTION);
        parser
    }

    pub fn parse(&mut self, argv: &[&str], mode: i32) {
        let argc = argv.len();
        // clear out possible previous parsing remnants
        self.flags.clear();
        self.params.clear();
        self.pos_args.clear();
        
        self.args.resize(argc, String::new());
        self.args.iter_mut()
            .zip(argv.iter())
            .for_each(|(arg, original)| *arg = original.to_string());

        // parse line
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
                    self.params.insert(
                        name[..equal_pos].to_string(),
                        name[equal_pos + 1..].to_string(),
                    );
                    i += 1;
                    continue;
                }
            }

            if self.args[i].len() == 1 + name.len()
                && (mode & Self::SINGLE_DASH_IS_MULTIFLAG != 0)
                && !self.is_param(&name)
            {
                let keep_param = if !name.is_empty() && self.is_param(&name[name.len() - 1..]) {
                    let param_part = name.pop().unwrap();
                    Some(param_part.to_string())
                } else {
                    None
                };

                for c in name.chars() {
                    self.flags.insert(c.to_string());
                }

                if let Some(param) = keep_param {
                    name = param;
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

            let prefer_param = mode & Self::PREFER_PARAM_FOR_UNREG_OPTION != 0;

            if self.is_param(&name) || prefer_param {
                self.params.insert(name, self.args[i + 1].clone());
                i += 2;
            } else {
                self.flags.insert(name);
                i += 1;
            }
        }
    }

    fn bad_stream(&self) -> String {
        "".to_string()
    }

    fn is_number(&self, arg: &str) -> bool {
        arg.parse::<f64>().is_ok()
    }

    fn is_option(&self, arg: &str) -> bool {
        assert!(!arg.is_empty());
        if self.is_number(arg) {
            return false;
        }
        arg.starts_with('-')
    }

    fn trim_leading_dashes(&self, name: &str) -> String {
        name.trim_start_matches('-').to_string()
    }

    fn got_flag(&self, name: &str) -> bool {
        self.flags.contains(&self.trim_leading_dashes(name))
    }

    fn is_param(&self, name: &str) -> bool {
        self.registered_params.contains(name)
    }

    pub fn add_param(&mut self, name: &str) {
        self.registered_params.insert(self.trim_leading_dashes(name));
    }

    pub fn flags(&self) -> &HashSet<String> {
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

    pub fn params_by_name<'a>(&'a self, name: &str) -> MultimapIterationWrapper<'a, String, String> {
        let trimmed_name = self.trim_leading_dashes(name);
        MultimapIterationWrapper::new(&self.params, &trimmed_name)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            args: Vec::new(),
            params: BTreeMap::new(),
            pos_args: Vec::new(),
            flags: HashSet::new(),
            registered_params: HashSet::new(),
            empty: String::new(),
        }
    }
}

impl std::ops::Index<&str> for Parser {
    type Output = bool;

    fn index(&self, name: &str) -> &Self::Output {
        static TRUE: bool = true;
        static FALSE: bool = false;

        if self.got_flag(name) {
            &TRUE
        } else {
            &FALSE
        }
    }
}

impl std::ops::Index<usize> for Parser {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.pos_args.len() {
            &self.pos_args[index]
        } else {
            &self.empty
        }
    }
}

impl Parser {
    pub fn get_flag(&self, names: &[&str]) -> bool {
        names.iter().any(|name| self.got_flag(name))
    }

    pub fn get_param_stream(&self, name: &str) -> String {
        if let Some(value) = self.params.get(&self.trim_leading_dashes(name)) {
            return value.clone();
        }
        self.bad_stream()
    }

    pub fn get_param_stream_with_default<T>(&self, name: &str, default: T) -> String
    where
        T: ToString,
    {
        if let Some(value) = self.params.get(&self.trim_leading_dashes(name)) {
            return value.clone();
        }
        default.to_string()
    }

    pub fn get_positional(&self, index: usize) -> String {
        self.pos_args.get(index).cloned().unwrap_or_else(|| self.bad_stream())
    }

    pub fn get_positional_with_default<T>(&self, index: usize, default: T) -> String
    where
        T: ToString,
    {
        self.pos_args.get(index).cloned().unwrap_or_else(|| default.to_string())
    }
}

fn main() {
    // This is a placeholder for the `main` function
    println!("Parser example");
}