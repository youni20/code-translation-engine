use std::io;

fn two_sum(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    println!("What two numbers would you like to add?");
    println!("Number 1: ");

    let mut x: i32 = 0;
    io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");

    println!("Number 2: ");

    let mut y: i32 = 0;
    io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");

    let result = two_sum(x, y);
    println!("The Result: {}", result);
}