use std::io::{self, Write};

fn two_sum(x: i32, y: i32) -> i32 {
    let result = x + y;
    result
}

fn main() {
    let mut x: i32 = 0;
    let mut y: i32 = 0;

    print!("What two numbers would you like to add?\nNumber 1: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    x = input.trim().parse().unwrap();

    print!("Number 2: ");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    y = input.trim().parse().unwrap();

    println!();

    let result = two_sum(x, y);
    println!("The Result: {}", result);
}