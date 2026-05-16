// sha1.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

/// compute SHA1 hash
/// Usage:
/// let mut sha1 = SHA1::new();
/// let my_hash = sha1.hash("Hello World".as_bytes()); // &[u8]
/// let my_hash2 = sha1.hash("How are you".as_bytes()); // arbitrary data, 11 bytes
///
/// // or in a streaming fashion:
///
/// let mut sha1 = SHA1::new();
/// while more_data_available {
///     sha1.add(pointer_to_fresh_data, number_of_new_bytes);
/// }
/// let my_hash3 = sha1.get_hash();
pub struct SHA1 {
    /// split into 64 byte blocks (=> 512 bits), hash is 20 bytes long
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; Self::BLOCK_SIZE],
    m_hash: [u32; Self::HASH_VALUES],
}

impl SHA1 {
    const BLOCK_SIZE: usize = 512 / 8;
    const HASH_BYTES: usize = 20;
    const HASH_VALUES: usize = Self::HASH_BYTES / 4;

    /// same as reset()
    pub fn new() -> Self {
        let mut sha = SHA1 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; Self::BLOCK_SIZE],
            m_hash: [0; Self::HASH_VALUES],
        };
        sha.reset();
        sha
    }

    /// compute SHA1 of a memory block
    pub fn hash(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // implementation omitted for brevity
    }

    /// return latest hash as 40 hex characters
    pub fn get_hash(&mut self) -> String {
        let mut buffer = [0u8; Self::HASH_BYTES];
        self.get_hash_bytes(&mut buffer);
        buffer.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// return latest hash as bytes
    pub fn get_hash_bytes(&mut self, _buffer: &mut [u8; Self::HASH_BYTES]) {
        self.process_buffer();
        // implementation omitted for brevity
    }

    /// restart
    pub fn reset(&mut self) {
        self.m_num_bytes = 0;
        self.m_buffer_size = 0;
        self.m_hash = [
            0x67452301,
            0xEFCDAB89,
            0x98BADCFE,
            0x10325476,
            0xC3D2E1F0,
        ];
    }

    /// process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // implementation omitted for brevity
    }

    /// process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // implementation omitted for brevity
    }
}

fn main() {
    // Basic usage examples to ensure compilation
    let mut sha1 = SHA1::new();
    let _ = sha1.hash("Hello World".as_bytes());
}