use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let argv: Vec<&str> = args.iter().map(|s| &s[..]).collect();

    let mut verbose = false;

    for arg in &argv {
        if *arg == "-v" {
            verbose = true;
            break; 
        }
    }

    if verbose {
        println!("Verbose, I am.");
    }
}