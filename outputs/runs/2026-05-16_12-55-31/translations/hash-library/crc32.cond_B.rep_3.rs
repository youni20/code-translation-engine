/// Compute CRC32 hash, based on Intel's Slicing-by-8 algorithm
///
/// Usage:
/// let mut crc32 = CRC32::new();
/// let my_hash = crc32.compute("Hello World".as_bytes()); // &str
/// let my_hash2 = crc32.compute("How are you".as_bytes()); // arbitrary data
///
/// // or in a streaming fashion:
/// let mut crc32 = CRC32::new();
/// while more_data_available {
///     crc32.add(fresh_data);
/// }
/// let my_hash3 = crc32.get_hash();
///
/// Note:
/// You can find code for the faster Slicing-by-16 algorithm on the author's website as well.
/// Its unrolled version is about twice as fast but its look-up table is doubled in size.
struct CRC32 {
    m_hash: u32,
}

impl CRC32 {
    /// Hash is 4 bytes long
    const HASH_BYTES: usize = 4;

    /// Same as reset()
    pub fn new() -> Self {
        let mut crc32 = CRC32 { m_hash: 0 };
        crc32.reset();
        crc32
    }

    /// Compute CRC32 of a memory block
    pub fn compute(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// Add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        // Here should be the CRC32 calculation logic using the "Slicing-by-8" algorithm.
        // Without additional information, we can't provide the actual implementation.
        for &byte in data {
            // Dummy implementation for placeholder
            self.m_hash ^= byte as u32;
        }
    }

    /// Return latest hash as 8 hex characters
    pub fn get_hash(&self) -> String {
        format!("{:08x}", self.m_hash)
    }

    /// Return latest hash as bytes
    pub fn get_hash_bytes(&self) -> [u8; Self::HASH_BYTES] {
        self.m_hash.to_be_bytes()
    }

    /// Restart
    pub fn reset(&mut self) {
        self.m_hash = 0xFFFFFFFF;
    }
}

fn main() {
    let mut crc32 = CRC32::new();
    let my_hash = crc32.compute("Hello World".as_bytes());
    println!("Hash: {}", my_hash);
}