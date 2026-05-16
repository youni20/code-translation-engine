use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-v".to_string()) {
        println!("Verbose, I am.");
    }
}