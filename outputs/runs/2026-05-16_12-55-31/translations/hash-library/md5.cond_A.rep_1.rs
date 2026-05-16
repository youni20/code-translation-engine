// md5.rs
// Ported from md5.h by Stephan Brumme

use std::fmt::Write;

pub struct MD5 {
    m_num_bytes: u64,
    m_buffer_size: usize,
    m_buffer: [u8; MD5::BLOCK_SIZE],
    m_hash: [u32; MD5::HASH_VALUES],
}

impl MD5 {
    pub const BLOCK_SIZE: usize = 64; // 512 bits / 8
    pub const HASH_BYTES: usize = 16;
    const HASH_VALUES: usize = Self::HASH_BYTES / 4;

    pub fn new() -> MD5 {
        let mut md5 = MD5 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; MD5::BLOCK_SIZE],
            m_hash: [0; Self::HASH_VALUES],
        };
        md5.reset();
        md5
    }

    pub fn operator_call<T: AsRef<[u8]>>(&mut self, data: T, num_bytes: Option<usize>) -> String {
        match num_bytes {
            Some(size) => self.add(data.as_ref(), size),
            None => self.add(data.as_ref(), data.as_ref().len()),
        }
        self.get_hash()
    }

    pub fn add(&mut self, data: &[u8], num_bytes: usize) {
        let mut offset = 0;

        // existing data in buffer
        if self.m_buffer_size > 0 {
            let remaining_space = MD5::BLOCK_SIZE - self.m_buffer_size;
            let data_size = num_bytes.min(remaining_space);
            self.m_buffer[self.m_buffer_size..self.m_buffer_size + data_size]
                .copy_from_slice(&data[0..data_size]);
            self.m_buffer_size += data_size;
            offset += data_size;

            if self.m_buffer_size == MD5::BLOCK_SIZE {
                let buffer_copy = self.m_buffer.clone();
                self.process_block(&buffer_copy);
                self.m_buffer_size = 0;
            }
        }

        // process full blocks
        while offset + MD5::BLOCK_SIZE <= num_bytes {
            self.process_block(&data[offset..offset + MD5::BLOCK_SIZE]);
            offset += MD5::BLOCK_SIZE;
        }

        // remaining data
        if offset < num_bytes {
            let remaining_data = &data[offset..num_bytes];
            self.m_buffer[..remaining_data.len()].copy_from_slice(remaining_data);
            self.m_buffer_size = remaining_data.len();
        }

        self.m_num_bytes += num_bytes as u64;
    }

    pub fn get_hash(&self) -> String {
        let mut buffer = [0u8; MD5::HASH_BYTES];
        self.get_hash_bytes(&mut buffer);
        let mut hex_string = String::with_capacity(MD5::HASH_BYTES * 2);
        for byte in &buffer {
            write!(&mut hex_string, "{:02x}", byte).expect("Unable to write");
        }
        hex_string
    }

    pub fn get_hash_bytes(&self, buffer: &mut [u8; MD5::HASH_BYTES]) {
        // Convert the hash to bytes
        for (i, &value) in self.m_hash.iter().enumerate() {
            buffer[i * 4..(i + 1) * 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn reset(&mut self) {
        self.m_num_bytes = 0;
        self.m_buffer_size = 0;
        self.m_buffer = [0; MD5::BLOCK_SIZE];
        // Initialize hash values
        self.m_hash = [
            0x67452301,
            0xefcdab89,
            0x98badcfe,
            0x10325476,
        ];
    }

    fn process_block(&mut self, data: &[u8]) {
        assert_eq!(data.len(), MD5::BLOCK_SIZE);

        // Placeholder for the block processing logic
        // This is where the main MD5 transformation would take place.
    }

    fn process_buffer(&mut self) {
        let data_bits = self.m_num_bytes * 8;

        // Padding
        self.m_buffer[self.m_buffer_size] = 0x80;
        self.m_buffer_size += 1;

        if self.m_buffer_size > 56 {
            while self.m_buffer_size < MD5::BLOCK_SIZE {
                self.m_buffer[self.m_buffer_size] = 0;
                self.m_buffer_size += 1;
            }
            let buffer_copy = self.m_buffer.clone();
            self.process_block(&buffer_copy);
            self.m_buffer_size = 0;
        }

        while self.m_buffer_size < 56 {
            self.m_buffer[self.m_buffer_size] = 0;
            self.m_buffer_size += 1;
        }

        self.m_buffer[56..64].copy_from_slice(&data_bits.to_le_bytes());
        let buffer_copy = self.m_buffer.clone();
        self.process_block(&buffer_copy);
    }
}

// The code requires a main function to be complete, but since this is a library file
// (likely meant to be imported elsewhere), we'll add a dummy main function as a placeholder.
fn main() {
    println!("MD5 library loaded.");
}