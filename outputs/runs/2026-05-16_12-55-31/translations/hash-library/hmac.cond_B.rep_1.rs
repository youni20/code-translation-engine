// //////////////////////////////////////////////////////////
// hmac.rs
// Translated to Rust based on Stephan Brumme's C++ original
//

// The HMAC implementation is based on: 
// http://tools.ietf.org/html/rfc2104 and 
// http://en.wikipedia.org/wiki/Hash-based_message_authentication_code

use std::vec::Vec;

/// Usage example in Rust:
///
/// let msg = "The quick brown fox jumps over the lazy dog";
/// let key = "key";
/// let md5hmac  = hmac::<MD5>(msg, key);
/// let sha1hmac = hmac::<SHA1>(msg, key);
/// let sha2hmac = hmac::<SHA256>(msg, key);

/// Computes the HMAC hash of data and key using a specified hash method
pub fn hmac<HashMethod: Hash>(data: &[u8], key: &[u8]) -> Vec<u8> {
    // initialize key with zeros
    let mut used_key = vec![0u8; HashMethod::BLOCK_SIZE];

    // adjust length of key: must contain exactly block_size bytes
    if key.len() <= HashMethod::BLOCK_SIZE {
        // copy key
        used_key[..key.len()].copy_from_slice(key);
    } else {
        // shorten key: used_key = hashed(key)
        let mut key_hasher = HashMethod::new();
        key_hasher.add(key);
        key_hasher.get_hash(&mut used_key);
    }

    // create initial XOR padding
    for i in 0..HashMethod::BLOCK_SIZE {
        used_key[i] ^= 0x36;
    }

    // inside = hash((used_key ^ 0x36) + data)
    let mut inside = vec![0u8; HashMethod::HASH_BYTES];
    let mut inside_hasher = HashMethod::new();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    inside_hasher.get_hash(&mut inside);

    // undo used_key's previous 0x36 XORing and apply a XOR by 0x5C
    for i in 0..HashMethod::BLOCK_SIZE {
        used_key[i] ^= 0x5C ^ 0x36;
    }

    // hash((used_key ^ 0x5C) + hash((used_key ^ 0x36) + data))
    let mut final_hasher = HashMethod::new();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    let mut result = vec![0u8; HashMethod::HASH_BYTES];
    final_hasher.get_hash(&mut result);

    result
}

/// Convenience function for &str usage
pub fn hmac_str<HashMethod: Hash>(data: &str, key: &str) -> Vec<u8> {
    hmac::<HashMethod>(data.as_bytes(), key.as_bytes())
}

pub trait Hash {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;

    fn new() -> Self;
    fn add(&mut self, data: &[u8]);
    fn get_hash(&mut self, hash: &mut [u8]);
}

// Example implementation stubs, these would need to be fully implemented
pub struct MD5;
impl Hash for MD5 {
    const BLOCK_SIZE: usize = 64;
    const HASH_BYTES: usize = 16;

    fn new() -> Self {
        MD5
    }
    fn add(&mut self, _data: &[u8]) {
        // Implementation for adding data to the MD5 hasher
    }
    fn get_hash(&mut self, _hash: &mut [u8]) {
        // Implementation for retrieving the MD5 hash
    }
}

pub struct SHA1;
impl Hash for SHA1 {
    const BLOCK_SIZE: usize = 64;
    const HASH_BYTES: usize = 20;

    fn new() -> Self {
        SHA1
    }
    fn add(&mut self, _data: &[u8]) {
        // Implementation for adding data to the SHA1 hasher
    }
    fn get_hash(&mut self, _hash: &mut [u8]) {
        // Implementation for retrieving the SHA1 hash
    }
}

pub struct SHA256;
impl Hash for SHA256 {
    const BLOCK_SIZE: usize = 64;
    const HASH_BYTES: usize = 32;

    fn new() -> Self {
        SHA256
    }
    fn add(&mut self, _data: &[u8]) {
        // Implementation for adding data to the SHA256 hasher
    }
    fn get_hash(&mut self, _hash: &mut [u8]) {
        // Implementation for retrieving the SHA256 hash
    }
}

fn main() {
    let msg = "The quick brown fox jumps over the lazy dog";
    let key = "key";
    let _md5hmac = hmac::<MD5>(msg.as_bytes(), key.as_bytes());
    let _sha1hmac = hmac::<SHA1>(msg.as_bytes(), key.as_bytes());
    let _sha2hmac = hmac::<SHA256>(msg.as_bytes(), key.as_bytes());

    // Further code for demonstration or testing could go here
}