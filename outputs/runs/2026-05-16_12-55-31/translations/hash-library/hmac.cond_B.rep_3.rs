/// Trait for hash methods that conform to the requirements for HMAC
pub trait HashMethod: Default {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;

    fn add(&mut self, data: &[u8]);
    fn get_hash(&self) -> Vec<u8>;
}

/// Compute HMAC hash of data and key using MD5, SHA1 or SHA256
pub fn hmac<Hash: HashMethod>(data: &[u8], key: &[u8]) -> String {
    // initialize key with zeros
    let mut used_key = vec![0u8; Hash::BLOCK_SIZE];

    // adjust length of key: must contain exactly BLOCK_SIZE bytes
    if key.len() <= Hash::BLOCK_SIZE {
        // copy key
        used_key[..key.len()].copy_from_slice(key);
    } else {
        // shorten key: used_key = hashed(key)
        let mut key_hasher = Hash::default();
        key_hasher.add(key);
        let key_hash = key_hasher.get_hash();
        used_key[..key_hash.len()].copy_from_slice(&key_hash);
    }

    // create initial XOR padding
    for i in 0..Hash::BLOCK_SIZE {
        used_key[i] ^= 0x36;
    }

    // inside = hash((used_key ^ 0x36) + data)
    let mut inside_hasher = Hash::default();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    let inside = inside_hasher.get_hash();

    // undo used_key's previous 0x36 XORing and apply a XOR by 0x5C
    for i in 0..Hash::BLOCK_SIZE {
        used_key[i] ^= 0x5C ^ 0x36;
    }

    // hash((used_key ^ 0x5C) + hash((used_key ^ 0x36) + data))
    let mut final_hasher = Hash::default();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);

    // simple hexadecimal string conversion
    final_hasher.get_hash().iter().map(|byte| format!("{:02x}", byte)).collect()
}

/// Convenience function for `String`
pub fn hmac_from_str<Hash: HashMethod>(data: &str, key: &str) -> String {
    hmac::<Hash>(data.as_bytes(), key.as_bytes())
}

fn main() {
    // Placeholder main function
}