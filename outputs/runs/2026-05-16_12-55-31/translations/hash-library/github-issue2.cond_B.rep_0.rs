use std::ffi::CString;
use std::os::raw::c_char;

// Assuming these modules are analogous to C++ counterpart;
// Dummy struct definitions for SHA1, SHA256, SHA3, Keccak, MD5
struct SHA1;
struct SHA256;
struct SHA3;
struct Keccak;
struct MD5;

impl SHA1 {
    fn new() -> Self {
        SHA1
    }

    fn add(&mut self, data: *const c_char, size: usize) {
        // Example of processing data; real implementation would handle it
    }

    fn get_hash(&self) -> String {
        // Dummy output; real implementation would return actual hash
        "fake_sha1_hash".to_string()
    }
}

impl SHA256 {
    fn new() -> Self {
        SHA256
    }

    fn add(&mut self, data: *const c_char, size: usize) {
        // Example of processing data; real implementation would handle it
    }

    fn get_hash(&self) -> String {
        // Dummy output; real implementation would return actual hash
        "fake_sha256_hash".to_string()
    }
}

impl SHA3 {
    fn new() -> Self {
        SHA3
    }

    fn add(&mut self, data: *const c_char, size: usize) {
        // Example of processing data; real implementation would handle it
    }

    fn get_hash(&self) -> String {
        // Dummy output; real implementation would return actual hash
        "fake_sha3_hash".to_string()
    }
}

impl Keccak {
    fn new() -> Self {
        Keccak
    }

    fn add(&mut self, data: *const c_char, size: usize) {
        // Example of processing data; real implementation would handle it
    }

    fn get_hash(&self) -> String {
        // Dummy output; real implementation would return actual hash
        "fake_keccak_hash".to_string()
    }
}

impl MD5 {
    fn new() -> Self {
        MD5
    }

    fn add(&mut self, data: *const c_char, size: usize) {
        // Example of processing data; real implementation would handle it
    }

    fn get_hash(&self) -> String {
        // Dummy output; real implementation would return actual hash
        "fake_md5_hash".to_string()
    }
}

fn main() {
    let text = "hello world";
    let c_text = CString::new(text).expect("CString::new failed");
    let c_str = c_text.as_ptr();

    println!("SHA1:");
    let mut sha1 = SHA1::new();
    sha1.add(c_str, text.len());
    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());

    println!("\nSHA256:");
    let mut sha256 = SHA256::new();
    sha256.add(c_str, text.len());
    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());

    println!("\nSHA3:");
    let mut sha3 = SHA3::new();
    sha3.add(c_str, text.len());
    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());

    println!("\nKeccak:");
    let mut keccak = Keccak::new();
    keccak.add(c_str, text.len());
    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());

    println!("\nMD5:");
    let mut md5 = MD5::new();
    md5.add(c_str, text.len());
    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
}