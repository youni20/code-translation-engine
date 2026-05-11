use std::io::{self, Write};

fn main() {
    let mut name = String::new();
    let mut age = String::new();

    print!("Enter your name: ");
    io::stdout().flush().unwrap(); // Flush stdout to ensure prompt appears before input
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim(); // Remove trailing newline

    print!("{}\nEnter your age: ", "\n"); // Print newline before entering age
    io::stdout().flush().unwrap(); // Flush stdout
    io::stdin().read_line(&mut age).expect("Failed to read line");
    let age: i32 = age.trim().parse().expect("Please enter a valid number");

    println!("\nWelcome {} You Are {} Years Old!", name, age);
}