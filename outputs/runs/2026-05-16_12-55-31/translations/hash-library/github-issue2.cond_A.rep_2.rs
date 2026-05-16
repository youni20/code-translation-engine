mod hash_library {
    pub mod sha1 {
        pub struct SHA1 {
            // Internal state goes here
        }

        impl SHA1 {
            pub fn new() -> Self {
                SHA1 {
                    // Initialize state
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Process data
            }

            pub fn get_hash(&self) -> String {
                // Return the hash as a hex string
                "fake_sha1_hash".to_string()
            }
        }
    }

    pub mod sha256 {
        pub struct SHA256 {
            // Internal state goes here
        }

        impl SHA256 {
            pub fn new() -> Self {
                SHA256 {
                    // Initialize state
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Process data
            }

            pub fn get_hash(&self) -> String {
                // Return the hash as a hex string
                "fake_sha256_hash".to_string()
            }
        }
    }

    pub mod sha3 {
        pub struct SHA3 {
            // Internal state goes here
        }

        impl SHA3 {
            pub fn new() -> Self {
                SHA3 {
                    // Initialize state
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Process data
            }

            pub fn get_hash(&self) -> String {
                // Return the hash as a hex string
                "fake_sha3_hash".to_string()
            }
        }
    }

    pub mod keccak {
        pub struct Keccak {
            // Internal state goes here
        }

        impl Keccak {
            pub fn new() -> Self {
                Keccak {
                    // Initialize state
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Process data
            }

            pub fn get_hash(&self) -> String {
                // Return the hash as a hex string
                "fake_keccak_hash".to_string()
            }
        }
    }

    pub mod md5 {
        pub struct MD5 {
            // Internal state goes here
        }

        impl MD5 {
            pub fn new() -> Self {
                MD5 {
                    // Initialize state
                }
            }

            pub fn add(&mut self, data: &[u8]) {
                // Process data
            }

            pub fn get_hash(&self) -> String {
                // Return the hash as a hex string
                "fake_md5_hash".to_string()
            }
        }
    }
}

use hash_library::sha1::SHA1;
use hash_library::sha256::SHA256;
use hash_library::sha3::SHA3;
use hash_library::keccak::Keccak;
use hash_library::md5::MD5;

fn main() {
    let text = "hello world";

    println!("SHA1:");
    let mut sha1 = SHA1::new();
    sha1.add(text.as_bytes());

    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());
    println!("{}", sha1.get_hash());

    println!();

    println!("SHA256:");
    let mut sha256 = SHA256::new();
    sha256.add(text.as_bytes());

    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());
    println!("{}", sha256.get_hash());

    println!();

    println!("SHA3:");
    let mut sha3 = SHA3::new();
    sha3.add(text.as_bytes());

    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());
    println!("{}", sha3.get_hash());

    println!();

    println!("Keccak:");
    let mut keccak = Keccak::new();
    keccak.add(text.as_bytes());

    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());
    println!("{}", keccak.get_hash());

    println!();

    println!("MD5:");
    let mut md5 = MD5::new();
    md5.add(text.as_bytes());

    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
    println!("{}", md5.get_hash());
}