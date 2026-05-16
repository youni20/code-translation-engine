use std::ffi::CString;
use std::os::raw::c_char;

// Corrected extern function declarations without linkage issues
#[link(name = "sha3")]
extern "C" {
    fn sha3_new(bits: u32) -> *mut u8;
    fn sha3_add(context: *mut u8, data: *const c_char, size: usize);
    fn sha3_get_hash(context: *mut u8) -> *const c_char;
    fn sha3_free(context: *mut u8);
}

fn main() {
    let text = "72a5f501151ca974002f34";

    unsafe {
        let hasher = sha3_new(512);

        if !hasher.is_null() {
            let c_text = CString::new(text).expect("CString::new failed");
            sha3_add(hasher, c_text.as_ptr(), text.len());

            let hash_ptr = sha3_get_hash(hasher);
            let hash = if !hash_ptr.is_null() {
                std::ffi::CStr::from_ptr(hash_ptr).to_string_lossy().into_owned()
            } else {
                String::new()
            };

            println!("{}", hash);

            sha3_free(hasher);
        }
    }
}