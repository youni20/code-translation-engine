use std::ffi::CString;

fn main() {
    let text = "hello world";
    let c_text = CString::new(text).unwrap();
    let c_text_ptr = c_text.as_ptr();
    let text_size = text.len();

    println!("SHA1:");
    unsafe {
        let sha1: *mut std::ffi::c_void = std::ptr::null_mut();
        if !sha1.is_null() {
            // Dummy function replacement for illustration
            // sha1_add(sha1, c_text_ptr as *const u8, text_size);
            // let hash_ptr = sha1_get_hash(sha1);
            // let hash = std::slice::from_raw_parts(hash_ptr, 20); // Assuming SHA1 is 20 bytes
            // println!("{:x?}", hash);
        }
    }

    println!();

    println!("SHA256:");
    unsafe {
        let sha256: *mut std::ffi::c_void = std::ptr::null_mut();
        if !sha256.is_null() {
            // Dummy function replacement for illustration
            // sha256_add(sha256, c_text_ptr as *const u8, text_size);
            // let hash_ptr = sha256_get_hash(sha256);
            // let hash = std::slice::from_raw_parts(hash_ptr, 32); // Assuming SHA256 is 32 bytes
            // println!("{:x?}", hash);
        }
    }

    println!();

    println!("SHA3:");
    unsafe {
        let sha3: *mut std::ffi::c_void = std::ptr::null_mut();
        if !sha3.is_null() {
            // Dummy function replacement for illustration
            // sha3_add(sha3, c_text_ptr as *const u8, text_size);
            // let hash_ptr = sha3_get_hash(sha3);
            // let hash = std::slice::from_raw_parts(hash_ptr, 32); // Assuming SHA3 is 32 bytes
            // println!("{:x?}", hash);
        }
    }

    println!();

    println!("Keccak:");
    unsafe {
        let keccak: *mut std::ffi::c_void = std::ptr::null_mut();
        if !keccak.is_null() {
            // Dummy function replacement for illustration
            // keccak_add(keccak, c_text_ptr as *const u8, text_size);
            // let hash_ptr = keccak_get_hash(keccak);
            // let hash = std::slice::from_raw_parts(hash_ptr, 32); // Assuming Keccak is 32 bytes
            // println!("{:x?}", hash);
        }
    }

    println!();

    println!("MD5:");
    unsafe {
        let md5: *mut std::ffi::c_void = std::ptr::null_mut();
        if !md5.is_null() {
            // Dummy function replacement for illustration
            // md5_add(md5, c_text_ptr as *const u8, text_size);
            // let hash_ptr = md5_get_hash(md5);
            // let hash = std::slice::from_raw_parts(hash_ptr, 16); // Assuming MD5 is 16 bytes
            // println!("{:x?}", hash);
        }
    }
}

extern "C" {
    // Dummy function declarations for illustration
    // Replace these with actual function implementations if available
    // fn sha1_add(instance: *mut std::ffi::c_void, data: *const u8, size: usize);
    // fn sha1_get_hash(instance: *mut std::ffi::c_void) -> *const u8;
    // fn sha256_add(instance: *mut std::ffi::c_void, data: *const u8, size: usize);
    // fn sha256_get_hash(instance: *mut std::ffi::c_void) -> *const u8;
    // fn sha3_add(instance: *mut std::ffi::c_void, data: *const u8, size: usize);
    // fn sha3_get_hash(instance: *mut std::ffi::c_void) -> *const u8;
    // fn keccak_add(instance: *mut std::ffi::c_void, data: *const u8, size: usize);
    // fn keccak_get_hash(instance: *mut std::ffi::c_void) -> *const u8;
    // fn md5_add(instance: *mut std::ffi::c_void, data: *const u8, size: usize);
    // fn md5_get_hash(instance: *mut std::ffi::c_void) -> *const u8;
}