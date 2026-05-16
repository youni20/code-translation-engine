use std::fmt;

pub struct CRC32 {
    // The hash is stored as a u32
    m_hash: u32,
}

impl CRC32 {
    pub const HASH_BYTES: usize = 4;

    pub fn new() -> CRC32 {
        let mut crc32 = CRC32 { m_hash: 0 };
        crc32.reset();
        crc32
    }

    pub fn call(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    pub fn call_str(&mut self, text: &str) -> String {
        self.call(text.as_bytes())
    }

    pub fn add(&mut self, data: &[u8]) {
        // Example CRC32 calculation, actual implementation omitted for brevity.
        // Implement the Slicing-by-8 or other algorithm here.
        for &byte in data {
            self.m_hash ^= byte as u32;
            // Placeholder for the CRC calculation logic:
            // m_hash = CRC32_TABLE[(m_hash ^ byte) & 0xFF] ^ (m_hash >> 8);
        }
    }

    pub fn get_hash(&self) -> String {
        format!("{:08x}", self.m_hash)
    }

    pub fn get_hash_bytes(&self, buffer: &mut [u8; Self::HASH_BYTES]) {
        let hash = self.m_hash.to_be_bytes();
        buffer.copy_from_slice(&hash);
    }

    pub fn reset(&mut self) {
        // Set to the initial CRC32 value
        self.m_hash = 0xFFFFFFFF;
    }
}

impl fmt::Display for CRC32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}", self.m_hash)
    }
}

fn main() {
    let mut crc = CRC32::new();
    let hash = crc.call_str("hello world");
    println!("CRC32 hash: {}", hash);
}