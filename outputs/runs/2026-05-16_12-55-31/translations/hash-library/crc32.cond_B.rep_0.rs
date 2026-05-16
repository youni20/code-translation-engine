use std::string::String;

pub struct CRC32 {
    hash: u32,
}

impl CRC32 {
    pub const HASH_BYTES: usize = 4;

    pub fn new() -> Self {
        let mut crc32 = CRC32 { hash: 0 };
        crc32.reset();
        crc32
    }

    pub fn apply(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    pub fn apply_from_string(&mut self, text: &str) -> String {
        self.apply(text.as_bytes())
    }

    pub fn add(&mut self, data: &[u8]) {
        for &byte in data {
            self.hash = (self.hash >> 8) ^ CRC32::TABLE[((self.hash as u8) ^ byte) as usize];
        }
    }

    pub fn get_hash(&self) -> String {
        format!("{:08x}", self.hash)
    }

    pub fn get_hash_bytes(&self, buffer: &mut [u8; Self::HASH_BYTES]) {
        let hash_bytes = self.hash.to_le_bytes();
        buffer.copy_from_slice(&hash_bytes[..Self::HASH_BYTES]);
    }

    pub fn reset(&mut self) {
        self.hash = 0xFFFFFFFF;
    }

    const TABLE: [u32; 256] = {
        const fn generate_table() -> [u32; 256] {
            let mut table = [0u32; 256];
            let mut i = 0;
            while i < 256 {
                let mut crc = i as u32;
                let polynomial = 0xEDB88320;
                let mut j = 0;
                while j < 8 {
                    if (crc & 1) != 0 {
                        crc = (crc >> 1) ^ polynomial;
                    } else {
                        crc >>= 1;
                    }
                    j += 1;
                }
                table[i] = crc;
                i += 1;
            }
            table
        }
        generate_table()
    };
}

fn main() {
    let mut crc32 = CRC32::new();
    let my_hash = crc32.apply_from_string("Hello World");
    println!("CRC32 Hash: {}", my_hash);

    crc32.reset();
    let data = b"How are you";
    let my_hash2 = crc32.apply(data);
    println!("CRC32 Hash: {}", my_hash2);

    crc32.reset();
    crc32.add(b"How ");
    crc32.add(b"are ");
    crc32.add(b"you ");
    let my_hash3 = crc32.get_hash();
    println!("CRC32 Hash: {}", my_hash3);
}