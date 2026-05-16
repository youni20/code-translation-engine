// crc32.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

use std::string::String;

pub struct CRC32 {
    m_hash: u32,
}

impl CRC32 {
    // Hash length in bytes
    const HASH_BYTES: usize = 4;

    // Create a new CRC32 with the hash reset
    pub fn new() -> CRC32 {
        let mut crc32 = CRC32 { m_hash: 0 };
        crc32.reset();
        crc32
    }

    // Compute CRC32 of a memory block given as a byte slice
    pub fn compute(&self, data: &[u8]) -> String {
        let mut crc32 = self.clone();
        crc32.add(data);
        crc32.get_hash()
    }

    // Compute CRC32 of a string, excluding the final zero
    pub fn compute_from_string(&self, text: &String) -> String {
        let bytes = text.as_bytes();
        self.compute(bytes)
    }

    // Add arbitrary number of bytes to the current checksum
    pub fn add(&mut self, _data: &[u8]) {
        // Compute CRC32 of the data and update self.m_hash
        // The actual CRC32 computation and table lookup would be implemented here.
    }

    // Return latest hash as 8 hex characters
    pub fn get_hash(&self) -> String {
        format!("{:08x}", self.m_hash)
    }

    // Return latest hash as bytes
    pub fn get_hash_bytes(&self, buffer: &mut [u8]) {
        assert!(buffer.len() >= CRC32::HASH_BYTES);
        for (i, byte) in self.m_hash.to_ne_bytes().iter().enumerate() {
            buffer[i] = *byte;
        }
    }

    // Restart the checksum calculation
    pub fn reset(&mut self) {
        self.m_hash = 0xFFFFFFFF; // Typically the initial CRC32 value
    }
}

impl Clone for CRC32 {
    fn clone(&self) -> Self {
        CRC32 { m_hash: self.m_hash }
    }
}

// main function for testing
fn main() {}