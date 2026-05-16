// //////////////////////////////////////////////////////////
// digest.rs
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

// rustc digest.rs

use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

// Dummy module implementations for crc32, md5, sha1, sha256, keccak, and sha3
// These should be replaced by actual implementations for the code to function
mod crc32 {
    pub struct CRC32;
    impl CRC32 {
        pub fn new() -> Self { CRC32 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { String::from("dummy_crc32") }
    }
}

mod md5 {
    pub struct MD5;
    impl MD5 {
        pub fn new() -> Self { MD5 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { String::from("dummy_md5") }
    }
}

mod sha1 {
    pub struct SHA1;
    impl SHA1 {
        pub fn new() -> Self { SHA1 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { String::from("dummy_sha1") }
    }
}

mod sha256 {
    pub struct SHA256;
    impl SHA256 {
        pub fn new() -> Self { SHA256 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { String::from("dummy_sha256") }
    }
}

mod keccak {
    pub struct Keccak;
    pub enum Variant {
        Keccak256,
    }
    impl Keccak {
        pub fn new(_variant: Variant) -> Self { Keccak }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { String::from("dummy_keccak") }
    }
}

mod sha3 {
    pub struct SHA3;
    pub enum Bits {
        Bits256,
    }
    impl SHA3 {
        pub fn new(_bits: Bits) -> Self { SHA3 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { String::from("dummy_sha3") }
    }
}

fn main() {
    // syntax check
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        println!("Usage: ./digest filename [--crc|--md5|--sha1|--sha256|--keccak|--sha3]");
        process::exit(1);
    }

    // parameters
    let filename = &args[1];
    let algorithm = if args.len() == 3 { &args[2] } else { "" };
    let compute_crc32 = algorithm.is_empty() || algorithm == "--crc";
    let compute_md5 = algorithm.is_empty() || algorithm == "--md5";
    let compute_sha1 = algorithm.is_empty() || algorithm == "--sha1";
    let compute_sha2 = algorithm.is_empty() || algorithm == "--sha2" || algorithm == "--sha256";
    let compute_keccak = algorithm.is_empty() || algorithm == "--keccak";
    let compute_sha3 = algorithm.is_empty() || algorithm == "--sha3";

    let mut digest_crc32 = crc32::CRC32::new();
    let mut digest_md5 = md5::MD5::new();
    let mut digest_sha1 = sha1::SHA1::new();
    let mut digest_sha2 = sha256::SHA256::new();
    let mut digest_keccak = keccak::Keccak::new(keccak::Variant::Keccak256);
    let mut digest_sha3 = sha3::SHA3::new(sha3::Bits::Bits256);

    // select input source: either file or standard-in
    let mut input: Box<dyn Read> = if filename == "-" {
        Box::new(io::stdin())
    } else {
        match File::open(filename) {
            Ok(file) => Box::new(file),
            Err(_) => {
                eprintln!("Can't open '{}'", filename);
                process::exit(2);
            }
        }
    };

    // each cycle processes about 1 MByte (divisible by 144 => improves Keccak/SHA3 performance)
    const BUFFER_SIZE: usize = 144 * 7 * 1024;
    let mut buffer = vec![0; BUFFER_SIZE];

    // process file
    loop {
        let num_bytes_read = match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

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

    // show results
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
}