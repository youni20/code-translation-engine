use std::env;

struct Argh {
    // In an actual translation, this would be more complex to handle various functionalities.
    positional_args: Vec<String>,
    flags: Vec<String>,
    params: Vec<(String, String)>,
    verbose: bool,
}

impl Argh {
    fn new(args: Vec<String>) -> Self {
        let mut positional_args = Vec::new();
        let mut flags = Vec::new();
        let mut params = Vec::new();
        let mut verbose = false;

        let mut args_iter = args.iter();
        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-v" => verbose = true,
                _ if arg.starts_with('-') => flags.push(arg.clone()),
                _ => positional_args.push(arg.clone()),
            }
        }

        Argh {
            positional_args,
            flags,
            params,
            verbose,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmdl = Argh::new(args);

    if cmdl.verbose {
        println!("Verbose, I am.");
    }

    println!("Positional args:");
    for pos_arg in &cmdl.positional_args {
        println!("\t{}", pos_arg);
    }

    println!("\nFlags:");
    for flag in &cmdl.flags {
        println!("\t{}", flag);
    }

    println!("\nParameters:");
    for (key, value) in &cmdl.params {
        println!("\t{} : {}", key, value);
    }
}