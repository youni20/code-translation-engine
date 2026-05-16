pub trait Hash {
    /// compute hash of a memory block
    fn hash_from_bytes(&mut self, data: &[u8]) -> String;
    /// compute hash of a string, excluding final zero
    fn hash_from_string(&mut self, text: &str) -> String;

    /// add arbitrary number of bytes
    fn add(&mut self, data: &[u8]);

    /// return latest hash as hex characters
    fn get_hash(&self) -> String;

    /// restart
    fn reset(&mut self);
}

fn main() {
    // Main function is empty because there is no usage specified.
}