// md5.rs
// Copyright (c) 2014 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

use std::string::String;

pub struct MD5 {
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; MD5::BLOCK_SIZE],
    m_hash: [u32; MD5::HASH_VALUES],
}

impl MD5 {
    /// Split into 64 byte blocks (=> 512 bits), hash is 16 bytes long
    const BLOCK_SIZE: usize = 512 / 8;
    const HASH_BYTES: usize = 16;
    const HASH_VALUES: usize = MD5::HASH_BYTES / 4;

    /// Same as reset()
    pub fn new() -> MD5 {
        let mut md5 = MD5 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; MD5::BLOCK_SIZE],
            m_hash: [0; MD5::HASH_VALUES],
        };
        md5.reset();
        md5
    }

    /// Compute MD5 of a memory block
    pub fn digest(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// Compute MD5 of a string, excluding final zero
    pub fn digest_str(&mut self, text: &str) -> String {
        self.digest(text.as_bytes())
    }

    /// Add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // Implementation will be added
    }

    /// Return latest hash as 32 hex characters
    pub fn get_hash(&self) -> String {
        let mut result = String::new();
        for &value in self.m_hash.iter() {
            result.push_str(&format!("{:08x}", value));
        }
        result
    }

    /// Return latest hash as bytes
    pub fn get_hash_bytes(&self) -> [u8; MD5::HASH_BYTES] {
        let buffer = [0u8; MD5::HASH_BYTES];
        // Implementation will be added
        buffer
    }

    /// Restart
    pub fn reset(&mut self) {
        // Initialization code will be added
    }

    /// Process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // Implementation will be added
    }

    /// Process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // Implementation will be added
    }
}

fn main() {
    // Your testing code here
}