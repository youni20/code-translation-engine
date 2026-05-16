// //////////////////////////////////////////////////////////
// hash.rs
// Converted from C++ to Rust
//

pub trait Hash {
    /// compute hash of a memory block
    fn compute_from_memory(&mut self, data: &[u8]) -> String;
    /// compute hash of a string, excluding final zero
    fn compute_from_string(&mut self, text: &str) -> String;

    /// add arbitrary number of bytes
    fn add(&mut self, data: &[u8]);

    /// return latest hash as hex characters
    fn get_hash(&self) -> String;

    /// restart
    fn reset(&mut self);
}

fn main() {
    // Example main function to satisfy the compiler.
    // The functionality to be added depends on how the trait Hash is used.
}