// //////////////////////////////////////////////////////////
// keccak.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

use std::vec::Vec;

/// compute Keccak hash (designated SHA3)
/** Usage:
    let mut keccak = Keccak::new(Bits::Keccak256);
    let my_hash = keccak("Hello World".as_bytes());     // Vec<u8>
    let my_hash2 = keccak(&"How are you".as_bytes()[..11]); // arbitrary data, 11 bytes

    // or in a streaming fashion:

    let mut keccak = Keccak::new(Bits::Keccak256);
    while more_data_available {
        keccak.add(pointer_to_fresh_data, number_of_new_bytes);
    }
    let my_hash3 = keccak.get_hash();
*/

#[allow(non_camel_case_types)]
type uint8_t = u8;
#[allow(non_camel_case_types)]
type uint64_t = u64;

pub struct Keccak {
    m_hash: [uint64_t; StateSize],
    m_num_bytes: uint64_t,
    m_block_size: usize,
    m_buffer_size: usize,
    m_buffer: [uint8_t; MaxBlockSize],
    m_bits: Bits,
}

/// algorithm variants
#[derive(Copy, Clone)]
pub enum Bits {
    Keccak224 = 224,
    Keccak256 = 256,
    Keccak384 = 384,
    Keccak512 = 512,
}

impl Keccak {
    /// same as reset()
    pub fn new(bits: Bits) -> Self {
        let mut keccak = Keccak {
            m_hash: [0u64; StateSize],
            m_num_bytes: 0,
            m_block_size: Self::calculate_block_size(bits),
            m_buffer_size: 0,
            m_buffer: [0u8; MaxBlockSize],
            m_bits: bits,
        };
        keccak.reset();
        keccak
    }

    fn calculate_block_size(bits: Bits) -> usize {
        200 - 2 * ((bits as usize) / 8)
    }

    /// compute hash of a memory block
    pub fn hash(&mut self, data: &[u8]) -> Vec<u8> {
        self.add(data);
        self.get_hash()
    }

    /// compute hash of a string, excluding final zero
    pub fn hash_string(&mut self, text: &str) -> Vec<u8> {
        self.hash(text.as_bytes())
    }

    /// add arbitrary number of bytes
    pub fn add(&mut self, _data: &[u8]) {
        // Implementation to be added
    }

    /// return latest hash as hex characters
    pub fn get_hash(&self) -> Vec<u8> {
        // Implementation to be added
        vec![]
    }

    /// restart
    pub fn reset(&mut self) {
        // Reset logic (to be implemented)
    }

    /// process a full block
    fn process_block(&mut self, _data: &[u8]) {
        // Implementation to be added
    }

    /// process everything left in the internal buffer
    fn process_buffer(&mut self) {
        // Implementation to be added
    }
}

/// Constants for the Keccak struct
const StateSize: usize = 1600 / (8 * 8);
const MaxBlockSize: usize = 200 - 2 * (224 / 8);

fn main() {
    // This is a placeholder for a main function to enable compilation.
}