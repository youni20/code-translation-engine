// minimal test case for hash functions issue reproduction

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    let text = "hello world";

    println!("Hash for 'hello world':");
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();
    println!("{}", hash);
}