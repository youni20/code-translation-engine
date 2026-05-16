use std::string::String;

/// compute SHA3 hash
/** Usage:
    let mut sha3 = SHA3::new(Bits::Bits256);
    let my_hash = sha3.hash(b"Hello World"); // Vec<u8>
    let my_hash2 = sha3.hash(b"How are you"); // Vec<u8>, 11 bytes

    // or in a streaming fashion:

    let mut sha3 = SHA3::new(Bits::Bits256);
    while more_data_available {
      sha3.add(fresh_data);
    }
    let my_hash3 = sha3.get_hash();
  */
pub struct SHA3 {
    m_hash: [u64; SHA3::STATE_SIZE],
    m_num_bytes: u64,
    m_block_size: usize,
    m_buffer_size: usize,
    m_buffer: [u8; SHA3::MAX_BLOCK_SIZE],
    m_bits: Bits,
}

#[derive(Clone, Copy)]
pub enum Bits {
    Bits224 = 224,
    Bits256 = 256,
    Bits384 = 384,
    Bits512 = 512,
}

impl SHA3 {
    const STATE_SIZE: usize = 1600 / (8 * 8);
    const MAX_BLOCK_SIZE: usize = 200 - 2 * (224 / 8);

    /// same as reset()
    pub fn new(bits: Bits) -> SHA3 {
        let mut sha3 = SHA3 {
            m_hash: [0u64; SHA3::STATE_SIZE],
            m_num_bytes: 0,
            m_block_size: 0,
            m_buffer_size: 0,
            m_buffer: [0u8; SHA3::MAX_BLOCK_SIZE],
            m_bits: bits,
        };
        sha3.reset();
        sha3
    }

    /// compute hash of a memory block
    pub fn hash(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // function body
    }

    /// return latest hash as hex characters
    pub fn get_hash(&self) -> String {
        // placeholder for demonstration
        "sha3_hash".to_string()
    }

    /// restart
    pub fn reset(&mut self) {
        self.m_num_bytes = 0;
        self.m_block_size = 0;
        self.m_buffer_size = 0;
        for value in self.m_hash.iter_mut() {
            *value = 0;
        }
        // set other necessary initial states
    }

    /// process a full block
    fn process_block(&mut self, _data: &[u8]) {
        // function body
    }

    /// process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // function body
    }
}

fn main() {
    // Example usage of the SHA3 struct
    let mut sha3 = SHA3::new(Bits::Bits256);
    let my_hash = sha3.hash(b"Hello World");
    println!("Hash: {}", my_hash);
}