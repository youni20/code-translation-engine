use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::str::FromStr;

type ParseResult<T> = Result<T, Box<dyn std::error::Error>>;

struct StringStream {
    stream: String,
    position: usize,
}

impl StringStream {
    fn new(value: &str) -> Self {
        Self {
            stream: value.to_string(),
            position: 0,
        }
    }

    fn setstate(&mut self, _state: std::io::Result<()>) {
        // Stub function to emulate C++ behavior
    }

    fn str(&self) -> &String {
        &self.stream
    }

    fn operator_bool(&self) -> bool {
        self.position < self.stream.len()
    }
}

impl<T: FromStr + Clone> std::ops::ShrAssign<&mut T> for StringStream {
    fn shr_assign(&mut self, rhs: &mut T) {
        *rhs = self.stream[self.position..].parse().unwrap_or_else(|_| {
            self.setstate(Err(std::io::Error::new(std::io::ErrorKind::Other, "parse error")));
            rhs.clone()
        });
    }
}

struct MultimapIterationWrapper<'a> {
    container: &'a BTreeMap<String, String>,
    lb: Option<std::collections::btree_map::Range<'a, String, String>>,
    ub: Option<std::collections::btree_map::Range<'a, String, String>>,
}

impl<'a> MultimapIterationWrapper<'a> {
    fn new(container: &'a BTreeMap<String, String>, lb: Option<std::collections::btree_map::Range<'a, String, String>>, ub: Option<std::collections::btree_map::Range<'a, String, String>>) -> Self {
        Self {
            container,
            lb,
            ub,
        }
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

impl Parser {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            params: BTreeMap::new(),
            pos_args: Vec::new(),
            flags: BTreeSet::new(),
            registered_params: HashSet::new(),
            empty: String::new(),
        }
    }

    pub fn add_param(&mut self, name: &str) {
        self.registered_params.insert(self.trim_leading_dashes(name));
    }

    pub fn add_params(&mut self, init_list: &[&str]) {
        for name in init_list {
            self.add_param(name);
        }
    }

    pub fn parse(&mut self, argv: &[&str], mode: i32) {
        self.flags.clear();
        self.params.clear();
        self.pos_args.clear();

        self.args = argv.iter().map(|&arg| arg.to_string()).collect();

        for i in 0..self.args.len() {
            if !self.is_option(&self.args[i]) {
                self.pos_args.push(self.args[i].clone());
                continue;
            }

            let mut name = self.trim_leading_dashes(&self.args[i]);

            if (mode & 0x01) == 0 {
                if let Some(equal_pos) = name.find('=') {
                    self.params.insert(
                        name[..equal_pos].to_string(),
                        name[equal_pos + 1..].to_string(),
                    );
                    continue;
                }
            }

            if self.args[i].len() == name.len() + 1
                && (mode & 0x02) != 0
                && !self.is_param(&name)
            {
                let mut keep_param = String::new();

                if !name.is_empty() && self.is_param(&name[name.len() - 1..name.len()]) {
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

            let prefer_param = (mode & 0x04) != 0;

            if self.is_param(&name) || prefer_param {
                self.params.insert(name, self.args[i + 1].clone());
            } else {
                self.flags.insert(name);
            }
        }
    }

    fn bad_stream(&self) -> StringStream {
        let mut bad = StringStream::new("");
        bad.setstate(Err(std::io::Error::new(std::io::ErrorKind::Other, "fail state")));
        bad
    }

    fn is_number(&self, arg: &str) -> bool {
        arg.parse::<f64>().is_ok()
    }

    fn is_option(&self, arg: &str) -> bool {
        !arg.is_empty() && !self.is_number(arg) && arg.starts_with('-')
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

    pub fn flags(&self) -> &BTreeSet<String> {
        &self.flags
    }

    pub fn params_map(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    pub fn pos_args(&self) -> &Vec<String> {
        &self.pos_args
    }

    pub fn params(&self, name: &str) -> MultimapIterationWrapper {
        let trimmed_name = self.trim_leading_dashes(name);
        let lb = self.params.range((std::ops::Bound::Included(trimmed_name.clone()), std::ops::Bound::Unbounded));
        let ub = self.params.range((std::ops::Bound::Excluded(trimmed_name), std::ops::Bound::Unbounded));
        MultimapIterationWrapper::new(&self.params, Some(lb), Some(ub))
    }

    pub fn size(&self) -> usize {
        self.pos_args.len()
    }
}

impl std::ops::Index<&str> for Parser {
    type Output = bool;

    fn index(&self, name: &str) -> &Self::Output {
        if self.got_flag(name) {
            return &true;
        }
        &false
    }
}

impl std::ops::Index<usize> for Parser {
    type Output = String;

    fn index(&self, ind: usize) -> &Self::Output {
        self.pos_args.get(ind).unwrap_or(&self.empty)
    }
}

impl Parser {
    pub fn operator(&self, index: usize) -> StringStream {
        if self.pos_args.len() <= index {
            return self.bad_stream();
        }

        StringStream::new(&self.pos_args[index])
    }

    pub fn operator_default<T: fmt::Display>(&self, index: usize, def_val: T) -> StringStream {
        if self.pos_args.len() <= index {
            let mut ostr = StringStream::new("");
            fmt::Write::write_fmt(&mut ostr.stream, format_args!("{}", def_val)).ok();
            return ostr;
        }

        StringStream::new(&self.pos_args[index])
    }

    pub fn parameter_stream(&self, name: &str) -> StringStream {
        if let Some(value) = self.params.get(&self.trim_leading_dashes(name)) {
            return StringStream::new(value);
        }

        self.bad_stream()
    }

    pub fn parameter_stream_default<T: fmt::Display>(
        &self,
        name: &str,
        def_val: T,
    ) -> StringStream {
        if let Some(value) = self.params.get(&self.trim_leading_dashes(name)) {
            return StringStream::new(value);
        }

        let mut ostr = StringStream::new("");
        fmt::Write::write_fmt(&mut ostr.stream, format_args!("{}", def_val)).ok();
        ostr
    }
}

fn main() {
    // Example usage
}