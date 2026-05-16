// //////////////////////////////////////////////////////////
// hmac.rs
// Translation of hmac.h from C++ to Rust by an expert code translator.
//
// Copyright (c) 2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

// based on http://tools.ietf.org/html/rfc2104
// see also http://en.wikipedia.org/wiki/Hash-based_message_authentication_code

use std::string::String;

/// compute HMAC hash of data and key using MD5, SHA1 or SHA256
pub fn hmac<HashMethod: Hash>(data: &[u8], key: &[u8]) -> String {
    // initialize key with zeros
    let mut used_key = vec![0u8; HashMethod::BLOCK_SIZE];

    // adjust length of key: must contain exactly blockSize bytes
    if key.len() <= HashMethod::BLOCK_SIZE {
        // copy key
        used_key[..key.len()].copy_from_slice(&key);
    } else {
        // shorten key: usedKey = hashed(key)
        let mut key_hasher = HashMethod::new();
        key_hasher.add(key);
        key_hasher.get_hash(&mut used_key);
    }

    // create initial XOR padding
    for i in 0..HashMethod::BLOCK_SIZE {
        used_key[i] ^= 0x36;
    }

    // inside = hash((usedKey ^ 0x36) + data)
    let mut inside = vec![0u8; HashMethod::HASH_BYTES];
    let mut inside_hasher = HashMethod::new();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    inside_hasher.get_hash(&mut inside);

    // undo usedKey's previous 0x36 XORing and apply a XOR by 0x5C
    for i in 0..HashMethod::BLOCK_SIZE {
        used_key[i] ^= 0x5C ^ 0x36;
    }

    // hash((usedKey ^ 0x5C) + hash((usedKey ^ 0x36) + data))
    let mut final_hasher = HashMethod::new();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    final_hasher.get_hash_as_string()
}

/// Trait that a hashing method must implement to be used with the hmac function.
pub trait Hash {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;

    fn new() -> Self;
    fn add(&mut self, data: &[u8]);
    fn get_hash(&self, hash: &mut [u8]);
    fn get_hash_as_string(&self) -> String;
}

/// Convenience function for `String`
pub fn hmac_string<HashMethod: Hash>(data: &str, key: &str) -> String {
    hmac::<HashMethod>(data.as_bytes(), key.as_bytes())
}

fn main() {
    // Example usage of the hmac function
    // Implement your own hash method and use it with `hmac` function as needed.
}