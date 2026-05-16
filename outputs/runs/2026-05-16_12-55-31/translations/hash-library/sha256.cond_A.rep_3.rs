use std::fmt::Write;

// Fixed size integer types are available directly in Rust
type Uint8T = u8;
type Uint32T = u32;
type Uint64T = u64;

pub struct SHA256 {
    m_num_bytes: Uint64T,
    m_buffer_size: usize,
    m_buffer: [Uint8T; SHA256::BLOCK_SIZE],
    m_hash: [Uint32T; SHA256::HASH_VALUES],
}

impl SHA256 {
    pub const BLOCK_SIZE: usize = 512 / 8;
    pub const HASH_BYTES: usize = 32;
    const HASH_VALUES: usize = Self::HASH_BYTES / 4;

    /// same as reset()
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

    /// compute SHA256 of a memory block
    pub fn operator(&mut self, data: &[u8]) -> String {
        self.reset();
        self.add(data);
        self.get_hash()
    }

    /// compute SHA256 of a string, excluding the final zero
    pub fn operator_string(&mut self, text: &str) -> String {
        self.operator(text.as_bytes())
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        let mut current_data = data;
        self.m_num_bytes += current_data.len() as Uint64T;

        if self.m_buffer_size > 0 {
            let available_space = Self::BLOCK_SIZE - self.m_buffer_size;
            let to_copy = available_space.min(current_data.len());
            self.m_buffer[self.m_buffer_size..self.m_buffer_size + to_copy]
                .copy_from_slice(&current_data[..to_copy]);
            self.m_buffer_size += to_copy;
            if self.m_buffer_size == Self::BLOCK_SIZE {
                let block = self.m_buffer;
                self.process_block(&block);
                self.m_buffer_size = 0;
            }
            current_data = &current_data[to_copy..];
        }

        while current_data.len() >= Self::BLOCK_SIZE {
            self.process_block(&current_data[..Self::BLOCK_SIZE]);
            current_data = &current_data[Self::BLOCK_SIZE..];
        }

        if !current_data.is_empty() {
            self.m_buffer[..current_data.len()].copy_from_slice(current_data);
            self.m_buffer_size = current_data.len();
        }
    }

    /// return latest hash as 64 hex characters
    pub fn get_hash(&self) -> String {
        let mut result = String::new();
        for &value in &self.m_hash {
            write!(result, "{:08x}", value).unwrap();
        }
        result
    }

    /// return latest hash as bytes
    pub fn get_hash_bytes(&self, buffer: &mut [u8; Self::HASH_BYTES]) {
        for (i, &value) in self.m_hash.iter().enumerate() {
            buffer[i * 4..(i + 1) * 4].copy_from_slice(&value.to_be_bytes());
        }
    }

    /// restart
    pub fn reset(&mut self) {
        self.m_num_bytes = 0;
        self.m_buffer_size = 0;
        self.m_hash = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
        ];
    }

    /// process 64 bytes
    fn process_block(&mut self, _data: &[u8]) {
        // SHA256 block processing logic
    }

    /// process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // SHA256 buffer processing logic
    }
}

fn main() {
    // Entry point for the program, can be filled with test or usage code
}