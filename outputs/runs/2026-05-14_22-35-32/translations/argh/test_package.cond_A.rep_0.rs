use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Assuming argh::parser::PREFER_PARAM_FOR_UNREG_OPTION allows unregistered options with parameter values.
    // In this Rust version, we assume all command-line arguments are potential options and parameters.
    let mut verbose_flag = false;

    for arg in &args {
        if arg == "-v" {
            verbose_flag = true;
            break;
        }
    }

    if verbose_flag {
        println!("Verbose, I am.");
    }
}