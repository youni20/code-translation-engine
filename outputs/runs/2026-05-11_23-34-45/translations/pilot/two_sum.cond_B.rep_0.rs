use std::io;

fn two_sum(x: i32, y: i32) -> i32 {
    let result = x + y;
    result
}

fn main() {
    let mut x = String::new();
    let mut y = String::new();
    println!("What two numbers would you like to add?");
    println!("Number 1: ");
    io::stdin().read_line(&mut x).expect("Failed to read line");
    
    println!("Number 2: ");
    io::stdin().read_line(&mut y).expect("Failed to read line");
    
    let x: i32 = x.trim().parse().expect("Please enter a valid number");
    let y: i32 = y.trim().parse().expect("Please enter a valid number");
    println!();
    
    let result = two_sum(x, y);
    println!("The Result: {}", result);
}