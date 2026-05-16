use std::string::String;

/// Compute SHA3 hash
/// Usage:
/// let mut sha3 = SHA3::new(SHA3Bits::Bits256);
/// let my_hash = sha3.hash("Hello World".as_bytes());
/// let my_hash2 = sha3.hash("How are you".as_bytes());
///
/// // or in a streaming fashion:
///
/// let mut sha3 = SHA3::new(SHA3Bits::Bits256);
/// while more_data_available {
///     sha3.add(pointer_to_fresh_data, number_of_new_bytes);
/// }
/// let my_hash3 = sha3.get_hash();

pub struct SHA3 {
    m_hash: [u64; SHA3::StateSize],
    m_num_bytes: u64,
    m_block_size: usize,
    m_buffer_size: usize,
    m_buffer: [u8; SHA3::MaxBlockSize],
    m_bits: SHA3Bits,
}

pub enum SHA3Bits {
    Bits224 = 224,
    Bits256 = 256,
    Bits384 = 384,
    Bits512 = 512,
}

impl SHA3 {
    const StateSize: usize = 1600 / (8 * 8);
    const MaxBlockSize: usize = 200 - 2 * (224 / 8);

    /// Same as reset()
    pub fn new(bits: SHA3Bits) -> Self {
        let mut instance = SHA3 {
            m_hash: [0; Self::StateSize],
            m_num_bytes: 0,
            m_block_size: 0,
            m_buffer_size: 0,
            m_buffer: [0; Self::MaxBlockSize],
            m_bits: bits,
        };
        instance.reset();
        instance
    }

    /// Compute hash of a memory block
    pub fn hash(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// Compute hash of a string
    pub fn hash_str(&mut self, text: &str) -> String {
        self.hash(text.as_bytes())
    }

    /// Add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        let mut input = data;
        if self.m_buffer_size > 0 {
            while self.m_buffer_size < self.m_block_size && !input.is_empty() {
                self.m_buffer[self.m_buffer_size] = input[0];
                self.m_buffer_size += 1;
                input = &input[1..];
            }
        }

        if self.m_buffer_size == self.m_block_size {
            let buffer_clone = self.m_buffer.clone();
            self.process_block(&buffer_clone);
            self.m_buffer_size = 0;
        }

        while input.len() >= self.m_block_size {
            self.process_block(&input[0..self.m_block_size]);
            input = &input[self.m_block_size..];
        }

        self.m_buffer[..input.len()].copy_from_slice(input);
        self.m_buffer_size = input.len();
    }

    /// Return latest hash as hex characters
    pub fn get_hash(&self) -> String {
        unimplemented!()
    }

    /// Restart
    pub fn reset(&mut self) {
        unimplemented!()
    }

    /// Process a full block
    fn process_block(&mut self, _data: &[u8]) {
        unimplemented!()
    }

    /// Process everything left in the internal buffer
    fn process_buffer(&mut self) {
        unimplemented!()
    }
}

fn main() {
    // Example usage of the SHA3 struct
}