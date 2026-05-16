// hmac.rs

/// A module to compute HMAC hashes using MD5, SHA1 or SHA256.

/// Usage example:
/// let msg = "The quick brown fox jumps over the lazy dog";
/// let key = "key";
/// let md5hmac = hmac::<MD5>(msg.as_bytes(), key.as_bytes());
/// let sha1hmac = hmac::<SHA1>(msg.as_bytes(), key.as_bytes());
/// let sha2hmac = hmac::<SHA256>(msg.as_bytes(), key.as_bytes());

/// Trait representing a hashing method required by HMAC
pub trait HashMethod {
    const BLOCK_SIZE: usize;
    const HASH_BYTES: usize;

    fn new() -> Self;
    fn add(&mut self, data: &[u8]);
    fn get_hash(&self, buffer: &mut [u8]);
}

/// Compute HMAC hash of data and key using a specified HashMethod
pub fn hmac<H: HashMethod>(data: &[u8], key: &[u8]) -> Vec<u8> {
    // Initialize key with zeros
    let mut used_key = vec![0u8; H::BLOCK_SIZE];

    // Adjust length of key: must contain exactly blockSize bytes
    if key.len() <= H::BLOCK_SIZE {
        // Copy key
        used_key[..key.len()].copy_from_slice(key);
    } else {
        // Shorten key: used_key = hashed(key)
        let mut key_hasher = H::new();
        key_hasher.add(key);
        key_hasher.get_hash(&mut used_key);
    }

    // Create initial XOR padding
    for byte in &mut used_key {
        *byte ^= 0x36;
    }

    // Inside = hash((used_key ^ 0x36) + data)
    let mut inside = vec![0u8; H::HASH_BYTES];
    let mut inside_hasher = H::new();
    inside_hasher.add(&used_key);
    inside_hasher.add(data);
    inside_hasher.get_hash(&mut inside);

    // Undo used_key's previous 0x36 XORing and apply a XOR by 0x5C
    for byte in &mut used_key {
        *byte ^= 0x5C ^ 0x36;
    }

    // Hash((used_key ^ 0x5C) + hash((used_key ^ 0x36) + data))
    let mut final_hasher = H::new();
    final_hasher.add(&used_key);
    final_hasher.add(&inside);
    final_hasher.get_hash(&mut inside);

    inside
}

// This main function is added to make the code compile as a standalone executable
fn main() {
    // Example usage can be placed here when implementing specific hash methods
}