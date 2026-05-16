// sha256.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

// use statements for needed Rust types
use std::fmt::Write;

pub struct SHA256 {
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; Self::BLOCK_SIZE],
    m_hash: [u32; Self::HASH_VALUES],
}

impl SHA256 {
    // split into 64 byte blocks (=> 512 bits), hash is 32 bytes long
    const BLOCK_SIZE: usize = 512 / 8;
    const HASH_BYTES: usize = 32;
    const HASH_VALUES: usize = Self::HASH_BYTES / 4;

    // same as reset()
    pub fn new() -> Self {
        let mut sha256 = SHA256 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0u8; Self::BLOCK_SIZE],
            m_hash: [0u32; Self::HASH_VALUES],
        };
        sha256.reset();
        sha256
    }

    // compute SHA256 of a memory block
    pub fn compute(&mut self, data: &[u8]) -> String {
        self.reset();
        self.add(data);
        self.get_hash()
    }

    // compute SHA256 of a string, excluding final zero
    pub fn from_string(&mut self, text: &str) -> String {
        self.compute(text.as_bytes())
    }

    // add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // (functionality to be implemented)
        // placeholder for adding data
    }

    // return latest hash as 64 hex characters
    pub fn get_hash(&self) -> String {
        let mut hash_string = String::with_capacity(SHA256::HASH_BYTES * 2);
        for &value in &self.m_hash {
            write!(&mut hash_string, "{:08x}", value).unwrap();
        }
        hash_string
    }

    // return latest hash as bytes
    pub fn get_hash_bytes(&self, buffer: &mut [u8; Self::HASH_BYTES]) {
        for (i, &value) in self.m_hash.iter().enumerate() {
            let bytes = value.to_be_bytes();
            buffer[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }
    }

    // restart
    pub fn reset(&mut self) {
        // (functionality to be implemented)
        // placeholder for resetting the state
    }

    // process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // (functionality to be implemented)
        // placeholder for processing a block
    }

    // process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // (functionality to be implemented)
        // placeholder for processing buffered data
    }
}

fn main() {
    let mut sha256 = SHA256::new();
    let hash1 = sha256.from_string("Hello World");
    println!("Hash1: {}", hash1);
    
    let data = b"How are you";
    let hash2 = sha256.compute(data);
    println!("Hash2: {}", hash2);
}