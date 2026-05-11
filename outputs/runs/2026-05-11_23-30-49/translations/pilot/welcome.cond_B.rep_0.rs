use std::io;

fn main() {
    let mut name = String::new();
    let mut age = String::new();

    println!("Enter your name: ");
    io::stdin().read_line(&mut name).expect("Failed to read line");
    let name = name.trim(); // Trim whitespace

    println!("\nEnter your age: ");
    io::stdin().read_line(&mut age).expect("Failed to read line");
    let age: i32 = age.trim().parse().expect("Please type a number!");

    println!("\nWelcome {}! You are {} years old!", name, age);
}