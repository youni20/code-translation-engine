// //////////////////////////////////////////////////////////
// hash.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

pub trait Hash {
    /// compute hash of a memory block
    fn compute(&mut self, data: &[u8]) -> String;
    /// compute hash of a string, excluding final zero
    fn compute_str(&mut self, text: &str) -> String;

    /// add arbitrary number of bytes
    fn add(&mut self, data: &[u8]);

    /// return latest hash as hex characters
    fn get_hash(&self) -> String;

    /// restart
    fn reset(&mut self);
}

fn main() {
    // Placeholder main function
}