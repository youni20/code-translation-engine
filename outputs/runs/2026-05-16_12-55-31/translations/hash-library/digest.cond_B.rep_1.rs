use std::env;
use std::fs::File;
use std::io::{self, Read};

// Assuming these imports are defined in their respective modules
// Placeholder definitions for the purpose of making code compile
mod crc32 {
    pub struct CRC32;
    impl CRC32 {
        pub fn new() -> Self { CRC32 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "crc32_hash".to_string() }
    }
}
mod md5 {
    pub struct MD5;
    impl MD5 {
        pub fn new() -> Self { MD5 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "md5_hash".to_string() }
    }
}
mod sha1 {
    pub struct SHA1;
    impl SHA1 {
        pub fn new() -> Self { SHA1 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "sha1_hash".to_string() }
    }
}
mod sha256 {
    pub struct SHA256;
    impl SHA256 {
        pub fn new() -> Self { SHA256 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "sha256_hash".to_string() }
    }
}
mod keccak {
    pub struct Keccak;
    impl Keccak {
        pub fn new(_variant: KeccakVariant) -> Self { Keccak }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "keccak_hash".to_string() }
    }
    pub enum KeccakVariant {
        Keccak256,
    }
}
mod sha3 {
    pub struct SHA3;
    impl SHA3 {
        pub fn new(_variant: SHA3Variant) -> Self { SHA3 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "sha3_hash".to_string() }
    }
    pub enum SHA3Variant {
        Bits256,
    }
}

use crate::crc32::CRC32;
use crate::md5::MD5;
use crate::sha1::SHA1;
use crate::sha256::SHA256;
use crate::keccak::{Keccak, KeccakVariant};
use crate::sha3::{SHA3, SHA3Variant};

fn main() -> io::Result<()> {
    // Collect arguments
    let args: Vec<String> = env::args().collect();

    // Syntax check
    if args.len() < 2 || args.len() > 3 {
        println!("./digest filename [--crc|--md5|--sha1|--sha256|--keccak|--sha3]");
        return Ok(());
    }

    // Parameters
    let filename = &args[1];
    let algorithm = if args.len() == 3 { &args[2] } else { "" };

    let compute_crc32 = algorithm.is_empty() || algorithm == "--crc";
    let compute_md5 = algorithm.is_empty() || algorithm == "--md5";
    let compute_sha1 = algorithm.is_empty() || algorithm == "--sha1";
    let compute_sha2 = algorithm.is_empty() || algorithm == "--sha2" || algorithm == "--sha256";
    let compute_keccak = algorithm.is_empty() || algorithm == "--keccak";
    let compute_sha3 = algorithm.is_empty() || algorithm == "--sha3";

    let mut digest_crc32 = CRC32::new();
    let mut digest_md5 = MD5::new();
    let mut digest_sha1 = SHA1::new();
    let mut digest_sha2 = SHA256::new();
    let mut digest_keccak = Keccak::new(KeccakVariant::Keccak256);
    let mut digest_sha3 = SHA3::new(SHA3Variant::Bits256);

    // Select input source: either file or standard-in
    let mut input: Box<dyn Read> = if filename == "-" {
        Box::new(io::stdin())
    } else {
        // Open file
        let file = File::open(filename)?;
        Box::new(file)
    };

    // Each cycle processes about 1 MByte (divisible by 144 => improves Keccak/SHA3 performance)
    const BUFFER_SIZE: usize = 144 * 7 * 1024;
    let mut buffer = vec![0; BUFFER_SIZE];

    // Process file
    loop {
        let num_bytes_read = input.read(&mut buffer)?;
        if num_bytes_read == 0 {
            break;
        }

        if compute_crc32 {
            digest_crc32.add(&buffer[..num_bytes_read]);
        }
        if compute_md5 {
            digest_md5.add(&buffer[..num_bytes_read]);
        }
        if compute_sha1 {
            digest_sha1.add(&buffer[..num_bytes_read]);
        }
        if compute_sha2 {
            digest_sha2.add(&buffer[..num_bytes_read]);
        }
        if compute_keccak {
            digest_keccak.add(&buffer[..num_bytes_read]);
        }
        if compute_sha3 {
            digest_sha3.add(&buffer[..num_bytes_read]);
        }
    }

    // Show results
    if compute_crc32 {
        println!("CRC32:      {}", digest_crc32.get_hash());
    }
    if compute_md5 {
        println!("MD5:        {}", digest_md5.get_hash());
    }
    if compute_sha1 {
        println!("SHA1:       {}", digest_sha1.get_hash());
    }
    if compute_sha2 {
        println!("SHA2/256:   {}", digest_sha2.get_hash());
    }
    if compute_keccak {
        println!("Keccak/256: {}", digest_keccak.get_hash());
    }
    if compute_sha3 {
        println!("SHA3/256:   {}", digest_sha3.get_hash());
    }

    Ok(())
}