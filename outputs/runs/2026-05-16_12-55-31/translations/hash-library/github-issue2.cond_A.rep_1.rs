// minimal test case for similar functionality in Rust
// Note: This assumes existence of Rust equivalents for the hash calculations
use std::io::{self, Write};

struct SHA1;
struct SHA256;
struct SHA3;
struct Keccak;
struct MD5;

// Placeholder implementations. Assume actual logic is implemented.
impl SHA1 {
    fn new() -> Self {
        SHA1
    }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> &str {
        "sha1hash"
    }
}

impl SHA256 {
    fn new() -> Self {
        SHA256
    }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> &str {
        "sha256hash"
    }
}

impl SHA3 {
    fn new() -> Self {
        SHA3
    }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> &str {
        "sha3hash"
    }
}

impl Keccak {
    fn new() -> Self {
        Keccak
    }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> &str {
        "keccakhash"
    }
}

impl MD5 {
    fn new() -> Self {
        MD5
    }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> &str {
        "md5hash"
    }
}

fn main() -> io::Result<()> {
    let text = "hello world";

    println!("SHA1:");
    let mut sha1 = SHA1::new();
    sha1.add(text.as_bytes());

    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());

    println!();

    println!("SHA256:");
    let mut sha256 = SHA256::new();
    sha256.add(text.as_bytes());

    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());

    println!();

    println!("SHA3:");
    let mut sha3 = SHA3::new();
    sha3.add(text.as_bytes());

    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());

    println!();

    println!("Keccak:");
    let mut keccak = Keccak::new();
    keccak.add(text.as_bytes());

    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());

    println!();

    println!("MD5:");
    let mut md5 = MD5::new();
    md5.add(text.as_bytes());

    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());

    Ok(())
}