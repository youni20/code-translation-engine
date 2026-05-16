pub trait Hash {
    /// compute hash of a memory block
    fn call(&mut self, data: &[u8]) -> String;
    
    /// compute hash of a string
    fn call_with_string(&mut self, text: &str) -> String;
    
    /// add arbitrary number of bytes
    fn add(&mut self, data: &[u8]);
    
    /// return latest hash as hex characters
    fn get_hash(&self) -> String;
    
    /// restart
    fn reset(&mut self);
}

fn main() {
    // This is a placeholder main function.
    // The actual logic for using the `Hash` trait needs to be implemented here.
}