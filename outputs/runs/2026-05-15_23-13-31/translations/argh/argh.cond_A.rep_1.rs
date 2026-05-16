use std::collections::BTreeMap;

pub mod argh {
    use std::str::FromStr;
    use std::collections::{BTreeMap, BTreeSet, HashSet};
    use std::usize;

    #[cfg(not(any(target_env = "gnu", target_env = "musl")))]
    pub type StringStream = std::io::Cursor<String>;

    #[cfg(any(target_env = "gnu", target_env = "musl"))]
    pub struct StringStream {
        inner: std::io::Cursor<String>,
    }

    #[cfg(any(target_env = "gnu", target_env = "musl"))]
    impl StringStream {
        pub fn new(s: String) -> Self {
            Self {
                inner: std::io::Cursor::new(s),
            }
        }

        pub fn bad() -> Self {
            Self {
                inner: std::io::Cursor::new(String::new()),
            }
        }

        pub fn read<T: FromStr>(&self) -> Option<T> {
            let s = self.inner.clone().into_inner();
            T::from_str(&s).ok()
        }

        pub fn setstate(&mut self) {
            // Set state to failed; in this case, we do nothing, as Rust handles errors with results and options.
        }

        pub fn str(&self) -> String {
            self.inner.clone().into_inner()
        }
    }

    #[cfg(any(target_env = "gnu", target_env = "musl"))]
    impl Clone for StringStream {
        fn clone(&self) -> Self {
            Self {
                inner: std::io::Cursor::new(self.str()),
            }
        }
    }

    pub struct MultiMapIterationWrapper<'a> {
        lb: std::collections::btree_map::Range<'a, String, String>,
        size: usize,
    }

    impl<'a> MultiMapIterationWrapper<'a> {
        pub fn new(range: std::collections::btree_map::Range<'a, String, String>) -> Self {
            let size = range.clone().count();
            MultiMapIterationWrapper { lb: range, size }
        }

        pub fn len(&self) -> usize {
            self.size
        }
    }

    impl<'a> IntoIterator for MultiMapIterationWrapper<'a> {
        type Item = (&'a String, &'a String);
        type IntoIter = std::collections::btree_map::Range<'a, String, String>;

        fn into_iter(self) -> Self::IntoIter {
            self.lb
        }
    }

    pub struct Parser {
        args: Vec<String>,
        params: BTreeMap<String, String>,
        pos_args: Vec<String>,
        flags: BTreeSet<String>,
        registered_params: HashSet<String>,
    }

    impl Parser {
        pub const PREFER_FLAG_FOR_UNREG_OPTION: i32 = 1 << 0;
        pub const PREFER_PARAM_FOR_UNREG_OPTION: i32 = 1 << 1;
        pub const NO_SPLIT_ON_EQUALSIGN: i32 = 1 << 2;
        pub const SINGLE_DASH_IS_MULTIFLAG: i32 = 1 << 3;

        pub fn new() -> Self {
            Parser {
                args: Vec::new(),
                params: BTreeMap::new(),
                pos_args: Vec::new(),
                flags: BTreeSet::new(),
                registered_params: HashSet::new(),
            }
        }

        pub fn parse(&mut self, argv: &[&str], mode: i32) {
            self.parse_args(argv.len(), argv, mode)
        }

        pub fn parse_args(&mut self, _argc: usize, argv: &[&str], mode: i32) {
            self.flags.clear();
            self.params.clear();
            self.pos_args.clear();

            self.args = argv.iter().map(|&arg| arg.to_owned()).collect();

            let mut i = 0;
            while i < self.args.len() {
                if !self.is_option(&self.args[i]) {
                    self.pos_args.push(self.args[i].clone());
                    i += 1;
                    continue;
                }

                let mut name = self.trim_leading_dashes(&self.args[i]);

                if (mode & Self::NO_SPLIT_ON_EQUALSIGN) == 0 {
                    if let Some(equal_pos) = name.find('=') {
                        self.params.insert(name[..equal_pos].to_string(), name[equal_pos + 1..].to_string());
                        i += 1;
                        continue;
                    }
                }

                if self.args[i].len() - name.len() == 1 && (mode & Self::SINGLE_DASH_IS_MULTIFLAG) != 0 && !self.is_param(&name) {
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
                        i += 1;
                        continue;
                    }
                }

                if i == self.args.len() - 1 || self.is_option(&self.args[i + 1]) {
                    self.flags.insert(name);
                    i += 1;
                    continue;
                }

                assert!((mode & Self::PREFER_FLAG_FOR_UNREG_OPTION) == 0 || (mode & Self::PREFER_PARAM_FOR_UNREG_OPTION) == 0);

                let prefer_param = (mode & Self::PREFER_PARAM_FOR_UNREG_OPTION) != 0;

                if self.is_param(&name) || prefer_param {
                    self.params.insert(name, self.args[i + 1].clone());
                    i += 2;
                } else {
                    self.flags.insert(name);
                    i += 1;
                }
            }
        }

        pub fn is_number(&self, arg: &str) -> bool {
            arg.parse::<f64>().is_ok()
        }

        pub fn is_option(&self, arg: &str) -> bool {
            !self.is_number(arg) && arg.starts_with('-')
        }

        pub fn trim_leading_dashes(&self, name: &str) -> String {
            name.trim_start_matches('-').to_owned()
        }

        pub fn got_flag(&self, name: &str) -> bool {
            self.flags.contains(&self.trim_leading_dashes(name))
        }

        pub fn is_param(&self, name: &str) -> bool {
            self.registered_params.contains(name)
        }

        pub fn add_param(&mut self, name: &str) {
            self.registered_params.insert(self.trim_leading_dashes(name));
        }

        pub fn add_params<T: AsRef<str>>(&mut self, names: &[T]) {
            for name in names {
                self.add_param(name.as_ref());
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

        pub fn params_for_name(&self, name: &str) -> MultiMapIterationWrapper {
            let trimmed_name = self.trim_leading_dashes(name);
            MultiMapIterationWrapper::new(self.params.range(trimmed_name.clone()..))
        }

        pub fn contains_flag(&self, name: &str) -> bool {
            self.got_flag(name)
        }

        pub fn contains_any_flag(&self, names: &[&str]) -> bool {
            names.iter().any(|&name| self.got_flag(name))
        }

        pub fn pos_arg(&self, index: usize) -> Option<&String> {
            self.pos_args.get(index)
        }

        pub fn param_value(&self, name: &str) -> Option<StringStream> {
            if let Some(value) = self.params.get(&self.trim_leading_dashes(name)) {
                Some(StringStream::new(value.clone()))
            } else {
                Some(StringStream::bad())
            }
        }

        pub fn param_value_with_default<T: ToString>(&self, name: &str, default: T) -> StringStream {
            if let Some(value) = self.params.get(&self.trim_leading_dashes(name)) {
                StringStream::new(value.clone())
            } else {
                StringStream::new(default.to_string())
            }
        }

        pub fn pos_arg_value(&self, index: usize) -> Option<StringStream> {
            self.pos_arg(index).map(|value| StringStream::new(value.clone())).or(Some(StringStream::bad()))
        }

        pub fn pos_arg_value_with_default<T: ToString>(&self, index: usize, default: T) -> StringStream {
            self.pos_arg(index).map(|value| StringStream::new(value.clone())).unwrap_or_else(|| StringStream::new(default.to_string()))
        }
    }
}

fn main() {}