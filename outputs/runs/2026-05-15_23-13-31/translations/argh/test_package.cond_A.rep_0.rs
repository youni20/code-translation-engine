use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let verbose = args.iter().any(|arg| arg == "-v");

    if verbose {
        println!("Verbose, I am.");
    }
}