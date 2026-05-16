use std::string::String;

/// compute HMAC hash of data and key using MD5, SHA1 or SHA256
pub fn hmac<HashMethod>(data: &[u8], key: &[u8]) -> String
where
    HashMethod: Default + HashMethodTrait,
{
    // initialize key with zeros
    let mut used_key = vec![0u8; HashMethod::BLOCK_SIZE];

    // adjust length of key: must contain exactly blockSize bytes
    if key.len() <= HashMethod::BLOCK_SIZE {
        // copy key
        used_key[..key.len()].copy_from_slice(key);
    } else {
        // shorten key: usedKey = hashed(key)
        let mut key_hasher = HashMethod::default();
        key_hasher.add(key);
        key_hasher.get_hash(&mut used_key[..HashMethod::HASH_BYTES]);
    }

    // create initial XOR padding
    for byte in &mut used_key {
        *byte ^= 0x36;
    }

    // inside = hash((usedKey ^ 0x36) + data)
    let mut inside = vec![0u8; HashMethod::HASH_BYTES];
    let mut inside_hasher = HashMethod::default();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    inside_hasher.get_hash(&mut inside[..]);

    // undo usedKey's previous 0x36 XORing and apply a XOR by 0x5C
    for byte in &mut used_key {
        *byte ^= 0x5C ^ 0x36;
    }

    // hash((usedKey ^ 0x5C) + hash((usedKey ^ 0x36) + data))
    let mut final_hasher = HashMethod::default();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    final_hasher.get_hash_string()
}

/// Trait defining the required methods and constants for a hash method
pub trait HashMethodTrait {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;

    fn add(&mut self, data: &[u8]);
    fn get_hash(&self, buffer: &mut [u8]);
    fn get_hash_string(self) -> String;
}

// A dummy main function to satisfy the compiler, replace it with your actual usage
fn main() {
    // Example instantiation of the hmac function here would require specific implementations of HashMethod.
}