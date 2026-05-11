use std::io;

fn main() {
    let mut name = String::new();
    let mut age = String::new();

    println!("Enter your name: ");
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim(); // Remove trailing newline

    println!("\nEnter your age: ");
    io::stdin().read_line(&mut age).expect("Failed to read line");
    let age: i32 = age.trim().parse().expect("Please enter a valid number");

    println!("\nWelcome {}! You Are {} Years Old!", name, age);
}