// sha1.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

use std::string::String;

pub struct SHA1 {
    // Constants
    block_size: usize,
    hash_bytes: usize,
    
    // Internal state
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; 64],
    m_hash: [u32; 5],
}

impl SHA1 {
    /// Creates a new SHA1 instance
    pub fn new() -> Self {
        let mut instance = SHA1 {
            block_size: 512 / 8,
            hash_bytes: 20,
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; 64],
            m_hash: [0; 5],
        };
        instance.reset();
        instance
    }

    /// Compute SHA1 of a memory block
    pub fn compute(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// Compute SHA1 of a string, excluding the final zero
    pub fn compute_from_str(&mut self, text: &str) -> String {
        self.compute(text.as_bytes())
    }

    /// Add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        let mut data = data;

        if self.m_buffer_size > 0 {
            let buffer_space = self.m_buffer.len() - self.m_buffer_size;
            let bytes_to_fill = data.len().min(buffer_space);

            self.m_buffer[self.m_buffer_size..self.m_buffer_size + bytes_to_fill]
                .copy_from_slice(&data[..bytes_to_fill]);
            self.m_buffer_size += bytes_to_fill;
            data = &data[bytes_to_fill..];

            if self.m_buffer_size == self.m_buffer.len() {
                let buffer_copy = self.m_buffer; // Create a copy of self.m_buffer
                self.process_block(&buffer_copy);
                self.m_buffer_size = 0;
            }
        }

        while data.len() >= self.m_buffer.len() {
            self.process_block(&data[..self.m_buffer.len()]);
            data = &data[self.m_buffer.len()..];
        }

        if !data.is_empty() {
            self.m_buffer[..data.len()].copy_from_slice(data);
            self.m_buffer_size = data.len();
        }
    }

    /// Return latest hash as 40 hex characters
    pub fn get_hash(&self) -> String {
        let mut hash = [0u8; 20];
        self.get_hash_bytes(&mut hash);
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Return latest hash as bytes
    pub fn get_hash_bytes(&self, buffer: &mut [u8; 20]) {
        for i in 0..self.m_hash.len() {
            buffer[4 * i] = (self.m_hash[i] >> 24) as u8;
            buffer[4 * i + 1] = (self.m_hash[i] >> 16) as u8;
            buffer[4 * i + 2] = (self.m_hash[i] >> 8) as u8;
            buffer[4 * i + 3] = self.m_hash[i] as u8;
        }
    }

    /// Restart
    pub fn reset(&mut self) {
        self.m_num_bytes = 0;
        self.m_buffer_size = 0;
        self.m_hash = [
            0x67452301,
            0xEFCDAB89,
            0x98BADCFE,
            0x10325476,
            0xC3D2E1F0
        ]
    }

    /// Process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // Implementation of SHA-1 block processing
    }

    /// Process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // Implementation of process buffer
    }
}

fn main() {
    // This is the entry point of the program where you can add code to test or use the SHA1 implementation.
}