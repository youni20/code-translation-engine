use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&String::from("-v")) {
        println!("Verbose, I am.");
    }
}