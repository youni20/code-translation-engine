use std::fmt::Write;

/// Compute HMAC hash of data and key using a hash method such as MD5, SHA1, or SHA256
/// Note: HashMethod must implement certain traits described below

pub trait HashMethod {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;
    
    fn new() -> Self where Self: Sized;
    fn add(&mut self, data: &[u8]);
    fn get_hash(&self, buffer: &mut [u8]);
}

fn hmac<HashMethodTy: HashMethod>(data: &[u8], key: &[u8]) -> String {
    // Use a vector to accommodate variable length arrays
    let mut used_key = vec![0u8; HashMethodTy::BLOCK_SIZE];

    // Adjust length of key: must contain exactly block size bytes
    if key.len() <= HashMethodTy::BLOCK_SIZE {
        // Copy key
        used_key[..key.len()].copy_from_slice(key);
    } else {
        // Shorten key: used_key = hashed(key)
        let mut key_hasher = HashMethodTy::new();
        key_hasher.add(key);
        key_hasher.get_hash(&mut used_key);
    }

    // Create initial XOR padding
    for byte in &mut used_key {
        *byte ^= 0x36;
    }

    // Inside = hash((used_key ^ 0x36) + data)
    let mut inside = vec![0u8; HashMethodTy::HASH_BYTES];
    let mut inside_hasher = HashMethodTy::new();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    inside_hasher.get_hash(&mut inside);

    // Undo used_key's previous 0x36 XORing and apply a XOR by 0x5C
    for byte in &mut used_key {
        *byte ^= 0x5C ^ 0x36;
    }

    // Hash((used_key ^ 0x5C) + hash((used_key ^ 0x36) + data))
    let mut final_hasher = HashMethodTy::new();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    let mut result = vec![0u8; HashMethodTy::HASH_BYTES];
    final_hasher.get_hash(&mut result);

    let mut hex_result = String::new();
    for byte in &result {
        write!(&mut hex_result, "{:02x}", byte).unwrap();
    }

    hex_result
}

/// Convenience function for String inputs
fn hmac_from_strings<HashMethodTy: HashMethod>(data: &str, key: &str) -> String {
    hmac::<HashMethodTy>(data.as_bytes(), key.as_bytes())
}

// Add a main function to serve as an entry point
fn main() {
    // This is just a placeholder example
    // Here you can call hmac_from_strings or hmac with specific data and key
}