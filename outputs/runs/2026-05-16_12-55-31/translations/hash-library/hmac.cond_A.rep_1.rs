// //////////////////////////////////////////////////////////
// hmac.rs
// Translation of Stephan Brumme's HMAC C++ code to Rust
// see http://create.stephan-brumme.com/disclaimer.html

// based on http://tools.ietf.org/html/rfc2104
// see also http://en.wikipedia.org/wiki/Hash-based_message_authentication_code

use std::vec::Vec;

/// Compute HMAC hash of data and key using a specified hash method
/// HashMethod trait must be implemented with the following requirements:
/// - const BLOCK_SIZE: usize;   // typically 64
/// - const HASH_BYTES: usize;   // e.g. length of hash in bytes, e.g. 20 for SHA1
/// - fn add(&mut self, data: &[u8]);
/// - fn get_hash(&self, dst: &mut [u8]);

pub trait HashMethod {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;
    fn add(&mut self, data: &[u8]);
    fn get_hash(&self, dst: &mut [u8]);
}

pub fn hmac<Hash: HashMethod + Default>(data: &[u8], key: &[u8]) -> Vec<u8> {
    // initialize key with zeros
    let mut used_key = vec![0u8; Hash::BLOCK_SIZE];

    // adjust length of key: must contain exactly block_size bytes
    if key.len() <= Hash::BLOCK_SIZE {
        used_key[..key.len()].copy_from_slice(key);
    } else {
        // shorten key: used_key = hashed(key)
        let mut key_hasher = Hash::default();
        key_hasher.add(key);
        key_hasher.get_hash(&mut used_key);
    }

    // create initial XOR padding
    for byte in &mut used_key {
        *byte ^= 0x36;
    }

    // inside = hash((used_key ^ 0x36) + data)
    let mut inside = vec![0u8; Hash::HASH_BYTES];
    let mut inside_hasher = Hash::default();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    inside_hasher.get_hash(&mut inside);

    // undo used_key's previous 0x36 XORing and apply a XOR by 0x5C
    for byte in &mut used_key {
        *byte ^= 0x36 ^ 0x5C;
    }

    // hash((used_key ^ 0x5C) + hash((used_key ^ 0x36) + data))
    let mut final_hasher = Hash::default();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    let mut final_hash = vec![0u8; Hash::HASH_BYTES];
    final_hasher.get_hash(&mut final_hash);

    final_hash
}

/// Convenience function for &str, which converts them to bytes.
///
/// Requires the use of a concrete implementation of the HashMethod trait.
pub fn hmac_from_str<Hash: HashMethod + Default>(data: &str, key: &str) -> Vec<u8> {
    hmac::<Hash>(data.as_bytes(), key.as_bytes())
}

// Entry point for testing purposes
fn main() {
    // Test implementation can be done here
}