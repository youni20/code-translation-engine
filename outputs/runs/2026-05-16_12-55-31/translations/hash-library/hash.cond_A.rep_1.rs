// //////////////////////////////////////////////////////////
// hash.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

use std::ffi::c_void;
use std::slice;

pub trait Hash {
    /// compute hash of a memory block
    fn compute_from_bytes(&mut self, data: *const c_void, num_bytes: usize) -> String;
    
    /// compute hash of a string, excluding final zero
    fn compute_from_string(&mut self, text: &str) -> String;

    /// add arbitrary number of bytes
    fn add(&mut self, data: *const c_void, num_bytes: usize);

    /// return latest hash as hex characters
    fn get_hash(&self) -> String;

    /// restart
    fn reset(&mut self);
}

// Helper function to convert *const c_void to a byte slice
fn ptr_to_byte_slice<'a>(data: *const c_void, num_bytes: usize) -> &'a [u8] {
    if data.is_null() || num_bytes == 0 {
        return &[];
    }
    unsafe { slice::from_raw_parts(data as *const u8, num_bytes) }
}

fn main() {
    // An example main function that does nothing
}