// //////////////////////////////////////////////////////////
// sha1.rs
// Ported from C++ by Stephan Brumme's SHA1 implementation

use std::fmt::Write;

/// SHA1 hashing implementation
pub struct SHA1 {
    // split into 64 byte blocks (=> 512 bits), hash is 20 bytes long
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; Self::BLOCK_SIZE],
    m_hash: [u32; Self::HASH_VALUES],
}

impl SHA1 {
    // Block and hash sizes
    const BLOCK_SIZE: usize = 512 / 8;
    const HASH_BYTES: usize = 20;
    const HASH_VALUES: usize = Self::HASH_BYTES / 4;

    /// same as reset()
    pub fn new() -> Self {
        let mut sha1 = SHA1 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; Self::BLOCK_SIZE],
            m_hash: [0; Self::HASH_VALUES],
        };
        sha1.reset();
        sha1
    }

    /// compute SHA1 of a memory block
    pub fn operator_with_data(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// compute SHA1 of a string, excluding final zero
    pub fn operator_with_string(&mut self, text: &str) -> String {
        self.operator_with_data(text.as_bytes())
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        let mut data = data;
        if data.len() == 0 { return; }

        // update size
        self.m_num_bytes += data.len() as u64;

        // process each block
        // fill the buffer first and process it
        if self.m_buffer_size > 0 {
            let size = self.m_buffer_size.min(Self::BLOCK_SIZE - self.m_buffer_size);
            self.m_buffer[self.m_buffer_size..self.m_buffer_size + size].copy_from_slice(&data[..size]);
            self.m_buffer_size += size;
            data = &data[size..];

            if self.m_buffer_size == Self::BLOCK_SIZE {
                let buffer = self.m_buffer.clone();
                self.process_block(&buffer);
                self.m_buffer_size = 0;
            }
        }

        // process full blocks directly from the input data
        while data.len() >= Self::BLOCK_SIZE {
            self.process_block(&data[..Self::BLOCK_SIZE]);
            data = &data[Self::BLOCK_SIZE..];
        }

        // store remaining bytes in buffer
        self.m_buffer[..data.len()].copy_from_slice(data);
        self.m_buffer_size = data.len();
    }

    /// return latest hash as 40 hex characters
    pub fn get_hash(&mut self) -> String {
        let mut hash_string = String::with_capacity(Self::HASH_BYTES * 2);
        let mut hash_value = [0u8; Self::HASH_BYTES];
        self.get_hash_bytes(&mut hash_value);
        for byte in &hash_value {
            write!(hash_string, "{:02x}", byte).unwrap();
        }
        hash_string
    }

    /// return latest hash as bytes
    pub fn get_hash_bytes(&mut self, buffer: &mut [u8; Self::HASH_BYTES]) {
        self.process_buffer();
        for (i, &val) in self.m_hash.iter().enumerate() {
            buffer[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
        }
    }

    /// restart
    pub fn reset(&mut self) {
        self.m_num_bytes = 0;
        self.m_buffer_size = 0;
        self.m_hash = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    }

    /// process 64 bytes
    fn process_block(&mut self, data: &[u8]) {
        assert!(data.len() == Self::BLOCK_SIZE);
        // Implement the actual SHA1 algorithm here...
    }

    /// process everything left in the internal buffer
    fn process_buffer(&mut self) {
        let padded_size = if self.m_buffer_size < 56 { 56 } else { 64 };
        let mut padding: Vec<u8> = vec![0; padded_size + 8];

        // padding: first bit set, others clear
        padding[0] = 0x80;

        // append length in bits
        let total_bits = self.m_num_bytes * 8;
        padding[padded_size..].copy_from_slice(&total_bits.to_be_bytes());

        // apply padding and process the remaining buffer
        self.add(&padding[..]);
    }
}

// Main function to ensure the code compiles
fn main() {
    let mut sha1 = SHA1::new();
    let hash = sha1.operator_with_string("hello");
    println!("{}", hash);
}