use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Convert args vector to slices for easier handling of command line parameters
    let args_slice: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if args_slice.contains(&"-v") {
        println!("Verbose, I am.");
    }
}