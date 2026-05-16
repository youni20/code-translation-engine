// //////////////////////////////////////////////////////////
// sha1.rs
// Based on C++ code by Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

use std::string::String;

pub struct SHA1 {
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; SHA1::BLOCK_SIZE],
    m_hash: [u32; SHA1::HASH_VALUES],
}

impl SHA1 {
    pub const BLOCK_SIZE: usize = 512 / 8;
    pub const HASH_BYTES: usize = 20;
    const HASH_VALUES: usize = SHA1::HASH_BYTES / 4;

    /// Constructs a new SHA1 hasher and resets its state.
    pub fn new() -> Self {
        let mut sha1 = SHA1 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; SHA1::BLOCK_SIZE],
            m_hash: [0; SHA1::HASH_VALUES],
        };
        sha1.reset();
        sha1
    }

    /// Compute SHA1 of a memory block
    pub fn compute(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// Compute SHA1 of a string
    pub fn compute_from_str(&mut self, text: &str) -> String {
        self.compute(text.as_bytes())
    }

    /// Add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // Your implementation will go here
    }

    /// Return latest hash as 40 hex characters
    pub fn get_hash(&mut self) -> String {
        // Your implementation will go here
        String::new()
    }

    /// Return latest hash as bytes
    pub fn get_hash_bytes(&mut self, _buffer: &mut [u8; SHA1::HASH_BYTES]) {
        // Your implementation will go here
    }

    /// Restart
    pub fn reset(&mut self) {
        // Your implementation will go here
    }

    /// Process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // Your implementation will go here
    }

    /// Process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // Your implementation will go here
    }
}

fn main() {
    // Example usage of SHA1
    let mut sha1 = SHA1::new();
    sha1.compute_from_str("test");
}