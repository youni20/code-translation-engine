// //////////////////////////////////////////////////////////
// sha1.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

use std::convert::TryInto;
use std::string::String;

/// SHA1 hashing struct
pub struct SHA1 {
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; SHA1::BLOCK_SIZE],
    m_hash: [u32; SHA1::HASH_VALUES],
}

impl SHA1 {
    /// split into 64 byte blocks (=> 512 bits), hash is 20 bytes long
    const BLOCK_SIZE: usize = 512 / 8;
    const HASH_BYTES: usize = 20;
    const HASH_VALUES: usize = SHA1::HASH_BYTES / 4;

    /// same as reset()
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

    /// compute SHA1 of a memory block
    pub fn compute_from_bytes(&mut self, data: &[u8]) -> String {
        self.reset();
        self.add(data);
        self.get_hash()
    }

    /// compute SHA1 of a string, excluding final zero
    pub fn compute_from_string(&mut self, text: &str) -> String {
        self.compute_from_bytes(text.as_bytes())
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        // Implementation here
    }

    /// return latest hash as 40 hex characters
    pub fn get_hash(&self) -> String {
        // Implementation to return hash as hex string
        String::new()
    }

    /// return latest hash as bytes
    pub fn get_hash_bytes(&self, buffer: &mut [u8; SHA1::HASH_BYTES]) {
        // Implementation to fill buffer with hash bytes
    }

    /// restart
    pub fn reset(&mut self) {
        // Implementation to reset
    }

    fn process_block(&mut self, data: &[u8]) {
        // Implementation
    }

    fn process_buffer(&mut self) {
        // Implementation
    }
}

fn main() {
    let mut sha1 = SHA1::new();
    let my_hash = sha1.compute_from_string("Hello World");
    println!("Hash: {}", my_hash);
}