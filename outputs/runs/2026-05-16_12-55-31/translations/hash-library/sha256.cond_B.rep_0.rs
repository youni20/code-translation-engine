type UInt8 = u8;
type UInt32 = u32;
type UInt64 = u64;

pub struct SHA256 {
    m_num_bytes: UInt64,
    m_buffer_size: usize,
    m_buffer: [UInt8; SHA256::BLOCK_SIZE as usize],
    m_hash: [UInt32; SHA256::HASH_VALUES],
}

impl SHA256 {
    pub const BLOCK_SIZE: usize = 64; // 512 bits
    pub const HASH_BYTES: usize = 32;
    const HASH_VALUES: usize = SHA256::HASH_BYTES / 4;

    pub fn new() -> SHA256 {
        let mut sha256 = SHA256 {
            m_num_bytes: 0,
            m_buffer_size: 0,
            m_buffer: [0; SHA256::BLOCK_SIZE],
            m_hash: [0; SHA256::HASH_VALUES],
        };
        sha256.reset();
        sha256
    }
    
    pub fn call(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    pub fn call_for_string(&mut self, text: &str) -> String {
        self.call(text.as_bytes())
    }

    pub fn add(&mut self, _data: &[u8]) {
        // add logic to add data to the current processing state
    }

    pub fn get_hash(&self) -> String {
        // compute hash as a hex string
        String::new()
    }

    pub fn get_hash_bytes(&self, _buffer: &mut [u8; SHA256::HASH_BYTES]) {
        // compute hash and fill buffer
    }

    pub fn reset(&mut self) {
        // reset the hash state
    }

    fn process_block(&mut self, _data: &[u8]) {
        // process a single block of 64 bytes
    }

    fn process_buffer(&mut self) {
        // process all remaining data in buffer
    }
}

fn main() {
    // Example usage code, if necessary, can be added here.
}