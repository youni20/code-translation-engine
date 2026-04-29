```rust
use std::io;

fn two_sum(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    let mut x: i32;
    let mut y: i32;

    println!("What two numbers would you like to add?");
    println!("Number 1: ");
    io::stdin()
        .read_line(&mut x.to_string())
        .expect("Failed to read line");

    println!("Number 2: ");
    io::stdin()
        .read_line(&mut y.to_string())
        .expect("Failed to read line");

    println!();

    let result = two_sum(x, y);
    println!("The Result: {}", result);
}
```