use std::collections::{BTreeMap, BTreeSet, VecDeque};

mod argh {
    use super::*;

    pub type Multimap<K, V> = BTreeMap<K, VecDeque<V>>;

    pub struct MultimapIterationWrapper<'a, K, V> {
        iterator: Box<dyn Iterator<Item = (&'a K, &'a V)> + 'a>,
        size: usize,
    }

    impl<'a, K, V> MultimapIterationWrapper<'a, K, V>
    where
        K: Ord + 'a,
    {
        fn new(map: &'a Multimap<K, V>, key: &'a K) -> Self {
            let size = map.get(key).map_or(0, |v| v.len());
            let iter = map
                .get(key)
                .into_iter()
                .flat_map(|v| v.iter())
                .map(move |v| (key, v));
            Self {
                iterator: Box::new(iter),
                size,
            }
        }

        pub fn size(&self) -> usize {
            self.size
        }
    }

    impl<'a, K, V> Iterator for MultimapIterationWrapper<'a, K, V> {
        type Item = (&'a K, &'a V);

        fn next(&mut self) -> Option<Self::Item> {
            self.iterator.next()
        }
    }

    #[derive(Default)]
    pub struct Parser {
        args: Vec<String>,
        params: Multimap<String, String>,
        pos_args: Vec<String>,
        flags: BTreeSet<String>,
        registered_params: BTreeSet<String>,
    }

    #[derive(Copy, Clone)]
    pub enum Mode {
        PreferFlagForUnregOption = 1 << 0,
        PreferParamForUnregOption = 1 << 1,
        NoSplitOnEqualSign = 1 << 2,
        SingleDashIsMultiflag = 1 << 3,
    }

    impl Parser {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_pre_reg_names(pre_reg_names: &[&str]) -> Self {
            let mut parser = Self::new();
            parser.add_params(pre_reg_names);
            parser
        }

        pub fn from_args(argv: &[&str], mode: Mode) -> Self {
            let mut parser = Self::default();
            parser.parse_args(argv, mode);
            parser
        }

        pub fn add_param(&mut self, name: &str) {
            self.registered_params.insert(Self::trim_leading_dashes(name));
        }

        pub fn add_params(&mut self, names: &[&str]) {
            for &name in names {
                self.add_param(name);
            }
        }

        pub fn parse_args(&mut self, argv: &[&str], mode: Mode) {
            self.flags.clear();
            self.params.clear();
            self.pos_args.clear();

            self.args = argv.iter().map(|&arg| arg.to_string()).collect();

            for i in 0..self.args.len() {
                if !self.is_option(&self.args[i]) {
                    self.pos_args.push(self.args[i].clone());
                    continue;
                }

                let mut name = Self::trim_leading_dashes(&self.args[i]);

                if !matches!(mode, Mode::NoSplitOnEqualSign) {
                    if let Some(equal_pos) = name.find('=') {
                        let key = name[0..equal_pos].to_string();
                        let value = name[(equal_pos + 1)..].to_string();
                        self.params.entry(key).or_default().push_back(value);
                        continue;
                    }
                }

                if self.args[i].len() - 1 == name.len()
                    && matches!(mode, Mode::SingleDashIsMultiflag)
                    && !self.is_param(&name)
                {
                    let mut keep_param = String::new();

                    if let Some(last) = name.pop() {
                        if self.is_param(&last.to_string()) {
                            keep_param.push(last);
                        }
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

                let prefer_param = matches!(mode, Mode::PreferParamForUnregOption);

                if self.is_param(&name) || prefer_param {
                    let next_val = self.args.get(i + 1).cloned().unwrap_or_default();
                    self.params.entry(name).or_default().push_back(next_val);
                } else {
                    self.flags.insert(name);
                }
            }
        }

        pub fn flags(&self) -> &BTreeSet<String> {
            &self.flags
        }

        pub fn params(&self) -> &Multimap<String, String> {
            &self.params
        }

        pub fn params_with_name(&self, name: &str) -> MultimapIterationWrapper<'_, String, String> {
            let key = Self::trim_leading_dashes(name).to_owned();
            MultimapIterationWrapper::new(&self.params, &key)
        }

        pub fn pos_args(&self) -> &[String] {
            &self.pos_args
        }

        pub fn size(&self) -> usize {
            self.pos_args.len()
        }

        pub fn contains_flag(&self, name: &str) -> bool {
            self.flags.contains(&Self::trim_leading_dashes(name))
        }

        fn is_option(&self, arg: &str) -> bool {
            !Parser::is_number(arg) && arg.starts_with('-')
        }

        fn is_param(&self, name: &str) -> bool {
            self.registered_params.contains(name)
        }

        fn trim_leading_dashes(name: &str) -> String {
            let pos = name.find(|c: char| c != '-');
            pos.map_or_else(|| "".to_string(), |p| name[p..].to_string())
        }

        fn is_number(arg: &str) -> bool {
            arg.parse::<f64>().is_ok()
        }

        pub fn get_flag(&self, name: &str) -> bool {
            self.contains_flag(name)
        }
    }
}

fn main() {
    // Example usage or tests can be written here
}