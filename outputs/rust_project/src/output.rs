use std::io::{self, Write};

fn two_sum(x: i32, y: i32) -> i32 {
    let result = x + y;
    result
}

fn main() {
    let mut x = String::new();
    let mut y = String::new();

    print!("What two numbers would you like to add?\nNumber 1: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut x).unwrap();

    print!("Number 2: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut y).unwrap();

    println!();

    let x: i32 = x.trim().parse().unwrap();
    let y: i32 = y.trim().parse().unwrap();

    let result = two_sum(x, y);
    println!("The Result: {}", result);
}