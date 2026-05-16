// md5.rs
// Copyright (c) 2014 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

/// Compute MD5 hash
/// Usage:
/// let mut md5 = MD5::new();
/// let my_hash = md5.compute("Hello World".as_bytes());
/// let my_hash2 = md5.compute(&"How are you".as_bytes()[0..11]);
///
/// // or in a streaming fashion:
///
/// while more data available {
///     md5.add(fresh_data, number_of_new_bytes);
/// }
/// let my_hash3 = md5.get_hash();

pub struct MD5 {
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; MD5::BLOCK_SIZE],
    m_hash: [u32; MD5::HASH_VALUES],
}

impl MD5 {
    // Split into 64 byte blocks (=> 512 bits), hash is 16 bytes long
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
    pub fn compute(&mut self, data: &[u8]) -> String {
        self.reset();
        self.add(data);
        self.get_hash()
    }

    /// Add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // Ongoing implementation for streaming data
    }

    /// Return latest hash as 32 hex characters
    pub fn get_hash(&self) -> String {
        // Result placeholder
        "hash_as_hex_string".to_string()
    }

    /// Return latest hash as bytes
    pub fn get_hash_bytes(&self, _buffer: &mut [u8; MD5::HASH_BYTES]) {
        // Hash bytes copy to buffer
    }

    /// Restart
    pub fn reset(&mut self) {
        // Reset implementation
    }

    /// Process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // Process block placeholder
    }

    /// Process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // Process buffer placeholder
    }
}

fn main() {
    // Example usage
    let mut md5 = MD5::new();
    let _hash = md5.compute("Hello World".as_bytes());
}