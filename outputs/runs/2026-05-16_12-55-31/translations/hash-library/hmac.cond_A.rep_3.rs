// //////////////////////////////////////////////////////////
// hmac.rs
// Copyright (c) 2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

// based on http://tools.ietf.org/html/rfc2104
// see also http://en.wikipedia.org/wiki/Hash-based_message_authentication_code

/// Usage:
/// let msg = "The quick brown fox jumps over the lazy dog";
/// let key = "key";
/// let md5hmac = hmac::<MD5>(msg, key);
/// let sha1hmac = hmac::<SHA1>(msg, key);
/// let sha2hmac = hmac::<SHA256>(msg, key);
///
/// Note:
/// To keep my code simple, HMAC computation currently needs the whole message at once.
/// This is in contrast to the hashes MD5, SHA1, etc. where an add() method is available
/// for incremental computation.
/// You can use any hash for HMAC as long as it provides:
/// - constant HashMethod::BLOCK_SIZE (typically 64)
/// - constant HashMethod::HASH_BYTES (length of hash in bytes, e.g. 20 for SHA1)
/// - HashMethod::add(buffer, buffer_size)
/// - HashMethod::get_hash() -> Vec<u8>

use std::marker::PhantomData;

/// compute HMAC hash of data and key using MD5, SHA1 or SHA256
fn hmac<HashMethod>(data: &[u8], key: &[u8]) -> Vec<u8>
where
    HashMethod: Hash + Default,
{
    let mut used_key = vec![0u8; HashMethod::BLOCK_SIZE];

    if key.len() <= HashMethod::BLOCK_SIZE {
        used_key[..key.len()].copy_from_slice(key);
    } else {
        let mut key_hasher: HashMethod = Default::default();
        key_hasher.add(key);
        used_key.copy_from_slice(&key_hasher.get_hash());
    }

    for i in 0..HashMethod::BLOCK_SIZE {
        used_key[i] ^= 0x36;
    }

    let mut inside_hasher: HashMethod = Default::default();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    let inside = inside_hasher.get_hash();

    for i in 0..HashMethod::BLOCK_SIZE {
        used_key[i] ^= 0x5C ^ 0x36;
    }

    let mut final_hasher: HashMethod = Default::default();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    final_hasher.get_hash()
}

/// convenience function for &str
fn hmac_str<HashMethod>(data: &str, key: &str) -> Vec<u8>
where
    HashMethod: Hash + Default,
{
    hmac::<HashMethod>(data.as_bytes(), key.as_bytes())
}

/// trait representing the necessary hash functions
trait Hash {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;

    fn add(&mut self, data: &[u8]);
    fn get_hash(&mut self) -> Vec<u8>;
}

fn main() {
    // Placeholder main function to ensure the file compiles;
    // replace it with actual usage, testing, or example code.
}