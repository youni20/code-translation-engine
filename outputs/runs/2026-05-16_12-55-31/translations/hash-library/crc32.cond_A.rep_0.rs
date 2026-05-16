// //////////////////////////////////////////////////////////
// crc32.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

use std::fmt;
use std::string::String;

/// compute CRC32 hash, based on Intel's Slicing-by-8 algorithm
/** Usage:
    let mut crc32 = CRC32::new();
    let my_hash = crc32.calculate("Hello World".as_bytes());     // &[u8]
    let my_hash2 = crc32.calculate("How are you".as_bytes()); // &[u8] of arbitrary data

    // or in a streaming fashion:

    let mut crc32 = CRC32::new();
    while more_data_available {
      crc32.add(pointer_to_fresh_data, number_of_new_bytes);
    }
    let my_hash3 = crc32.get_hash();
  */

pub struct CRC32 {
    /// hash
    m_hash: u32,
}

impl CRC32 {
    /// hash is 4 bytes long
    pub const HASH_BYTES: usize = 4;

    /// same as reset()
    pub fn new() -> Self {
        let mut crc32 = CRC32 {
            m_hash: 0xFFFFFFFF, // assuming reset should initialize it
        };
        crc32.reset();
        crc32
    }

    /// compute CRC32 of a memory block
    pub fn calculate(&mut self, data: &[u8]) -> String {
        self.add(data);
        self.get_hash()
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, data: &[u8]) {
        for &byte in data {
            let mut current_byte = byte as u32;
            for _ in 0..8 {
                if (self.m_hash ^ current_byte) & 1 != 0 {
                    self.m_hash = (self.m_hash >> 1) ^ 0xEDB88320;
                } else {
                    self.m_hash >>= 1;
                }
                current_byte >>= 1;
            }
        }
    }

    /// return latest hash as 8 hex characters
    pub fn get_hash(&self) -> String {
        format!("{:08x}", !self.m_hash)
    }

    /// return latest hash as bytes
    pub fn get_hash_bytes(&self) -> [u8; CRC32::HASH_BYTES] {
        (!self.m_hash).to_le_bytes()
    }

    /// restart
    pub fn reset(&mut self) {
        self.m_hash = 0xFFFFFFFF;
    }
}

impl fmt::Display for CRC32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_hash())
    }
}

fn main() {
    let mut crc32 = CRC32::new();
    let my_hash = crc32.calculate("Hello World".as_bytes());
    println!("CRC32 of 'Hello World': {}", my_hash);
}