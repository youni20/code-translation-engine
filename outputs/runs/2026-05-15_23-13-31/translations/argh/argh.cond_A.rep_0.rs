use std::collections::{HashSet, BTreeSet};

#[cfg(not(any(debug_assertions, feature = "nightly")))]
struct StringStreamProxy {
    stream: String,
    state: bool,
}

#[cfg(not(any(debug_assertions, feature = "nightly")))]
impl StringStreamProxy {
    fn new(value: &str) -> Self {
        Self {
            stream: value.to_string(),
            state: true,
        }
    }

    fn setstate(&mut self, state: bool) {
        self.state = state;
    }

    fn parse<T: std::str::FromStr>(&mut self) -> Option<T> {
        match self.stream.parse() {
            Ok(value) => {
                self.state = true;
                Some(value)
            }
            Err(_) => {
                self.state = false;
                None
            }
        }
    }

    fn str(&self) -> &str {
        &self.stream
    }

    fn is_good(&self) -> bool {
        self.state
    }
}

#[cfg(not(any(debug_assertions, feature = "nightly")))]
type StringStream = StringStreamProxy;

#[cfg(any(debug_assertions, feature = "nightly"))]
type StringStream = std::io::Cursor<String>;

struct MultiMapIterationWrapper<'a> {
    container: &'a Vec<(String, String)>,
    lb: usize,
    ub: usize,
}

impl<'a> Iterator for MultiMapIterationWrapper<'a> {
    type Item = (&'a String, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.lb < self.ub {
            let result = self.container.get(self.lb);
            self.lb += 1;
            result.map(|(k, v)| (k, v))
        } else {
            None
        }
    }
}

struct Parser {
    args: Vec<String>,
    params: Vec<(String, String)>,
    pos_args: Vec<String>,
    flags: BTreeSet<String>,
    registered_params: HashSet<String>,
    empty: String,
}

impl Parser {
    const PREFER_FLAG_FOR_UNREG_OPTION: i32 = 1 << 0;
    const PREFER_PARAM_FOR_UNREG_OPTION: i32 = 1 << 1;
    const NO_SPLIT_ON_EQUALSIGN: i32 = 1 << 2;
    const SINGLE_DASH_IS_MULTIFLAG: i32 = 1 << 3;

    fn new() -> Self {
        Self {
            args: Vec::new(),
            params: Vec::new(),
            pos_args: Vec::new(),
            flags: BTreeSet::new(),
            registered_params: HashSet::new(),
            empty: String::new(),
        }
    }

    fn add_param(&mut self, name: &str) {
        self.registered_params.insert(Self::trim_leading_dashes(name));
    }

    fn add_params(&mut self, names: &[&str]) {
        for &name in names {
            self.add_param(name);
        }
    }

    fn parse(&mut self, argv: &[&str], mode: i32) {
        self.flags.clear();
        self.params.clear();
        self.pos_args.clear();

        self.args = argv.iter().map(|s| (*s).to_string()).collect();

        let arg_count = self.args.len();

        let mut i = 0;
        while i < arg_count {
            if !Parser::is_option(&self.args[i]) {
                self.pos_args.push(self.args[i].clone());
                i += 1;
                continue;
            }

            let mut name = Self::trim_leading_dashes(&self.args[i]);

            if (mode & Self::NO_SPLIT_ON_EQUALSIGN) == 0 {
                if let Some(equal_pos) = name.find('=') {
                    let param_name = name[..equal_pos].to_string();
                    let param_value = name[equal_pos + 1..].to_string();
                    self.params.push((param_name, param_value));
                    i += 1;
                    continue;
                }
            }

            if self.args[i].len() == name.len() + 1
                && (mode & self.args[i].len() as i32) != 0
                && !self.is_param(&name)
            {
                let saved_param = name.pop().unwrap();
                for c in name.chars() {
                    self.flags.insert(c.to_string());
                }
                name = saved_param.to_string();
            }

            if i == arg_count - 1 || Self::is_option(&self.args[i + 1]) {
                self.flags.insert(name);
                i += 1;
                continue;
            }

            let prefer_param = (mode & Self::PREFER_PARAM_FOR_UNREG_OPTION) != 0;

            if self.is_param(&name) || prefer_param {
                self.params.push((name, self.args[i + 1].clone()));
                i += 2;
            } else {
                self.flags.insert(name);
                i += 1;
            }
        }
    }

    fn bad_stream() -> StringStream {
        let mut bad = StringStream::new("");
        bad.setstate(false);
        bad
    }

    fn is_number(arg: &str) -> bool {
        arg.parse::<f64>().is_ok()
    }

    fn is_option(arg: &str) -> bool {
        !arg.is_empty() && (arg.starts_with('-') && !Self::is_number(arg))
    }

    fn trim_leading_dashes(name: &str) -> String {
        name.trim_start_matches('-').to_string()
    }

    fn got_flag(&self, name: &str) -> bool {
        self.flags.contains(&Self::trim_leading_dashes(name))
    }

    fn is_param(&self, name: &str) -> bool {
        self.registered_params.contains(name)
    }

    fn size(&self) -> usize {
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

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.pos_args.len() {
            return &self.pos_args[index];
        }
        &self.empty
    }
}

impl Parser {
    fn operator(&self, name: &str) -> Option<StringStream> {
        let trimmed_name = Self::trim_leading_dashes(name);
        self.params
            .iter()
            .find(|&&(ref key, _)| *key == trimmed_name)
            .map(|&(_, ref value)| StringStream::new(value))
            .or_else(|| Some(Self::bad_stream()))
    }

    fn operator_with_init_list(&self, names: &[&str]) -> Option<StringStream> {
        for &name in names {
            if let Some(stream) = self.operator(name) {
                return Some(stream);
            }
        }
        Some(Self::bad_stream())
    }

    fn param_operator<T: ToString>(
        &self,
        name: &str,
        def_val: T,
    ) -> Option<StringStream> {
        let trimmed_name = Self::trim_leading_dashes(name);
        self.params
            .iter()
            .find(|&&(ref key, _)| *key == trimmed_name)
            .map(|&(_, ref value)| StringStream::new(value.to_string()))
            .or_else(move || Some(StringStream::new(def_val.to_string())))
    }

    fn param_operator_with_init_list<T: ToString>(
        &self,
        names: &[&str],
        def_val: T,
    ) -> Option<StringStream> {
        for &name in names {
            if let Some(stream) = self.param_operator(name, &def_val.to_string()) {
                return Some(stream);
            }
        }
        Some(StringStream::new(def_val.to_string()))
    }

    fn param_at(&self, index: usize) -> Option<StringStream> {
        if index < self.pos_args.len() {
            Some(StringStream::new(self.pos_args[index].to_string()))
        } else {
            Some(Self::bad_stream())
        }
    }

    fn param_at_default<T: ToString>(
        &self,
        index: usize,
        def_val: T,
    ) -> Option<StringStream> {
        if index < self.pos_args.len() {
            Some(StringStream::new(self.pos_args[index].to_string()))
        } else {
            Some(StringStream::new(def_val.to_string()))
        }
    }
}

fn main() {
    let mut parser = Parser::new();
    let args = vec!["-flag", "value", "--param=42"];
    parser.parse(&args, Parser::PREFER_FLAG_FOR_UNREG_OPTION);
    if parser["flag"] {
        println!("flag is set");
    }
    if let Some(mut stream) = parser.operator("param") {
        if let Some(value) = stream.parse::<i32>() {
            println!("param value: {}", value);
        }
    }
}