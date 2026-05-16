// minimal test case for https://github.com/stbrumme/hash-library/issues/2
// rustc github_issue2.rs && ./github_issue2

mod hash_library {
    pub mod sha1 {
        pub struct SHA1 {
            // Add necessary fields for SHA1
        }

        impl SHA1 {
            pub fn new() -> Self {
                SHA1 {
                    // Initialize fields
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Implement add method
            }

            pub fn get_hash(&self) -> String {
                // Implement get_hash method
                "".to_string()
            }
        }
    }

    pub mod sha256 {
        pub struct SHA256 {
            // Add necessary fields for SHA256
        }

        impl SHA256 {
            pub fn new() -> Self {
                SHA256 {
                    // Initialize fields
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Implement add method
            }

            pub fn get_hash(&self) -> String {
                // Implement get_hash method
                "".to_string()
            }
        }
    }

    pub mod sha3 {
        pub struct SHA3 {
            // Add necessary fields for SHA3
        }

        impl SHA3 {
            pub fn new() -> Self {
                SHA3 {
                    // Initialize fields
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Implement add method
            }

            pub fn get_hash(&self) -> String {
                // Implement get_hash method
                "".to_string()
            }
        }
    }

    pub mod keccak {
        pub struct Keccak {
            // Add necessary fields for Keccak
        }

        impl Keccak {
            pub fn new() -> Self {
                Keccak {
                    // Initialize fields
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Implement add method
            }

            pub fn get_hash(&self) -> String {
                // Implement get_hash method
                "".to_string()
            }
        }
    }

    pub mod md5 {
        pub struct MD5 {
            // Add necessary fields for MD5
        }

        impl MD5 {
            pub fn new() -> Self {
                MD5 {
                    // Initialize fields
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Implement add method
            }

            pub fn get_hash(&self) -> String {
                // Implement get_hash method
                "".to_string()
            }
        }
    }
}

use std::io::{self, Write};
use hash_library::sha1::SHA1;
use hash_library::sha256::SHA256;
use hash_library::sha3::SHA3;
use hash_library::keccak::Keccak;
use hash_library::md5::MD5;

fn main() {
    let text = "hello world".as_bytes();

    println!("SHA1:");
    let mut sha1 = SHA1::new();
    sha1.add(text);

    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());

    println!();

    println!("SHA256:");
    let mut sha256 = SHA256::new();
    sha256.add(text);

    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());

    println!();

    println!("SHA3:");
    let mut sha3 = SHA3::new();
    sha3.add(text);

    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());

    println!();

    println!("Keccak:");
    let mut keccak = Keccak::new();
    keccak.add(text);

    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());

    println!();

    println!("MD5:");
    let mut md5 = MD5::new();
    md5.add(text);

    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
}