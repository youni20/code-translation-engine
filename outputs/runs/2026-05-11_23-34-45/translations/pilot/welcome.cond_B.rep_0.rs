use std::io;
use std::io::Write;

fn main() {
    let mut name = String::new();
    let mut age = String::new();

    print!("Enter your name: ");
    io::stdout().flush().unwrap(); // Ensure prompt is displayed before reading input
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim(); // Remove any trailing newline

    print!("Enter your age: ");
    io::stdout().flush().unwrap(); // Ensure prompt is displayed before reading input
    io::stdin().read_line(&mut age).expect("Failed to read line");
    let age: i32 = age.trim().parse().expect("Please enter a valid number"); // Convert age to integer

    println!("\nWelcome {} You Are {} Years Old!", name, age);
}