use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut verbose = false;
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-v" => verbose = true,
            _ => {}
        }
    }

    if verbose {
        println!("Verbose, I am.");
    }
}