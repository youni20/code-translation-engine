use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod istringstream {
    use std::str::FromStr;
    use std::marker::PhantomData;

    pub struct IStringStream<'a> {
        content: &'a str,
        pos: usize,
        _marker: PhantomData<&'a ()>, // Adding PhantomData
    }

    impl<'a> IStringStream<'a> {
        pub fn new(content: &'a str) -> Self {
            Self { content, pos: 0, _marker: PhantomData }
        }

        pub fn eof(&self) -> bool {
            self.pos >= self.content.len()
        }

        pub fn get<T: FromStr>(&mut self) -> Option<T> {
            let end = self.content[self.pos..].find(char::is_whitespace).unwrap_or(0) + self.pos;
            let result = T::from_str(&self.content[self.pos..end]).ok();
            self.pos = end + 1; // move past the white space
            result
        }
    }

    impl<'a> PartialEq for IStringStream<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.content == other.content
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod istringstream {
    use std::str::FromStr;
    use std::marker::PhantomData;

    pub struct IStringStream<'a> {
        output: String,
        pos: usize,
        _marker: PhantomData<&'a ()>, // Adding PhantomData
    }

    impl<'a> IStringStream<'a> {
        pub fn new(content: &'a str) -> Self {
            let output = content.to_string();
            Self { output, pos: 0, _marker: PhantomData }
        }

        pub fn get<T: FromStr>(&mut self) -> Result<Option<T>, std::fmt::Error> {
            let remaining = &self.output[self.pos..];
            let end = remaining
                .find(char::is_whitespace)
                .unwrap_or_else(|| remaining.len())
                + self.pos;
            let value = &remaining[..end];
            let result = T::from_str(value).ok();
            self.pos = end + 1;
            if end < self.output.len() {
                Ok(result)
            } else {
                Err(std::fmt::Error)
            }
        }
    }

    impl<'a> PartialEq for IStringStream<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.output == other.output
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiMapIterationWrapper<'a> {
    container: &'a HashMap<String, Vec<String>>,
    lower_bound: usize,
    upper_bound: usize,
}

impl<'a> MultiMapIterationWrapper<'a> {
    pub fn new(
        container: &'a HashMap<String, Vec<String>>,
        lower_bound: usize,
        upper_bound: usize,
    ) -> Self {
        MultiMapIterationWrapper {
            container,
            lower_bound,
            upper_bound,
        }
    }

    pub fn size(&self) -> usize {
        self.upper_bound - self.lower_bound
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.container
            .values()
            .flat_map(|v| &v[self.lower_bound..self.upper_bound])
    }
}

pub enum Mode {
    PreferFlagForUnregOption = 1 << 0,
    PreferParamForUnregOption = 1 << 1,
    NoSplitOnEqualSign = 1 << 2,
    SingleDashIsMultiFlag = 1 << 3,
}

pub struct Parser {
    args: Vec<String>,
    params: HashMap<String, Vec<String>>,
    pos_args: Vec<String>,
    flags: HashSet<String>,
    registered_params: HashSet<String>,
    empty: String,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            params: HashMap::new(),
            pos_args: Vec::new(),
            flags: HashSet::new(),
            registered_params: HashSet::new(),
            empty: "".to_string(),
        }
    }

    pub fn new_with_pre_reg_names(pre_reg_names: &[&str]) -> Self {
        let mut parser = Self::new();
        parser.add_params(pre_reg_names);
        parser
    }

    pub fn new_with_args(argv: &[&str], mode: Mode) -> Self {
        let mut parser = Self::new();
        parser.parse(argv, mode as i32);
        parser
    }

    pub fn new_with_argc(argc: i32, argv: &[&str], mode: Mode) -> Self {
        let mut parser = Self::new();
        parser.parse_by_argc(argc, argv, mode as i32);
        parser
    }

    pub fn add_param(&mut self, name: &str) {
        self.registered_params.insert(self.trim_leading_dashes(name));
    }

    pub fn add_params(&mut self, init_list: &[&str]) {
        for &name in init_list {
            self.add_param(name);
        }
    }

    pub fn parse(&mut self, argv: &[&str], mode: i32) {
        let argc = argv.len() as i32;
        self.parse_by_argc(argc, argv, mode);
    }

    pub fn parse_by_argc(&mut self, argc: i32, argv: &[&str], mode: i32) {
        self.flags.clear();
        self.params.clear();
        self.pos_args.clear();

        self.args = argv.iter().map(|&s| s.to_string()).collect();

        for i in 0..argc as usize {
            if !self.is_option(&self.args[i]) {
                self.pos_args.push(self.args[i].clone());
                continue;
            }

            let mut name = self.trim_leading_dashes(&self.args[i]);

            if (mode & Mode::NoSplitOnEqualSign as i32) == 0 {
                if let Some(eq_pos) = name.find('=') {
                    self.insert_param(
                        name[..eq_pos].to_string(),
                        name[eq_pos + 1..].to_string(),
                    );
                    continue;
                }
            }

            if (self.args[i].len() - name.len() == 1)
                && ((mode & Mode::SingleDashIsMultiFlag as i32) != 0)
                && !self.is_param(&name)
            {
                let mut keep_param = String::new();

                if !name.is_empty() && self.is_param(&name[name.len() - 1..]) {
                    keep_param.push(name.pop().unwrap());
                }

                for c in name.chars() {
                    self.flags.insert(c.to_string());
                }

                if !keep_param.is_empty() {
                    name = keep_param;
                } else {
                    continue;
                }
            }

            if i == self.args.len() - 1 || self.is_option(&self.args[i + 1]) {
                self.flags.insert(name);
                continue;
            }

            let prefer_param = (mode & Mode::PreferParamForUnregOption as i32) != 0;

            if self.is_param(&name) || prefer_param {
                self.insert_param(name, self.args[i + 1].clone());
                continue;
            } else {
                self.flags.insert(name);
            }
        }
    }

    pub fn flags(&self) -> &HashSet<String> {
        &self.flags
    }

    pub fn params(&self) -> &HashMap<String, Vec<String>> {
        &self.params
    }

    pub fn params_for_name(&self, name: &str) -> MultiMapIterationWrapper {
        let name = self.trim_leading_dashes(name);
        let lower_bound = self
            .params
            .get(&name)
            .map_or(0, |v| self.params.len() - v.len());
        let upper_bound = self.params.len();
        MultiMapIterationWrapper::new(&self.params, lower_bound, upper_bound)
    }

    pub fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    pub fn bad_stream() -> self::istringstream::IStringStream<'static> {
        self::istringstream::IStringStream::new("")
    }

    pub fn is_number(&self, arg: &str) -> bool {
        arg.parse::<f64>().is_ok()
    }

    pub fn is_option(&self, arg: &str) -> bool {
        !arg.is_empty() && arg.starts_with('-') && !self.is_number(arg)
    }

    pub fn trim_leading_dashes(&self, name: &str) -> String {
        name.trim_start_matches('-').to_string()
    }

    pub fn got_flag(&self, name: &str) -> bool {
        self.flags.contains(&self.trim_leading_dashes(name))
    }

    pub fn is_param(&self, name: &str) -> bool {
        self.registered_params.contains(name)
    }

    fn insert_param(&mut self, name: String, value: String) {
        match self.params.entry(name) {
            Entry::Occupied(mut entry) => entry.get_mut().push(value),
            Entry::Vacant(entry) => {
                entry.insert(vec![value]);
            }
        }
    }
}

impl std::ops::Index<&str> for Parser {
    type Output = bool;

    fn index(&self, name: &str) -> &Self::Output {
        if self.got_flag(name) {
            &true
        } else {
            &false
        }
    }
}

impl std::ops::Index<usize> for Parser {
    type Output = String;

    fn index(&self, ind: usize) -> &Self::Output {
        self.pos_args.get(ind).unwrap_or(&self.empty)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl<'a> istringstream::IStringStream<'a> {
    pub fn new_with_default(content: &'a str, default: &'a str) -> Self {
        if content.is_empty() {
            Self::new(default)
        } else {
            Self::new(content)
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl<'a> istringstream::IStringStream<'a> {
    pub fn new_with_default(content: &'a str, default: &'a str) -> Self {
        if content.is_empty() {
            Self::new(default)
        } else {
            Self::new(content)
        }
    }
}

fn main() {
    // Main function added to allow the crate to compile.
}