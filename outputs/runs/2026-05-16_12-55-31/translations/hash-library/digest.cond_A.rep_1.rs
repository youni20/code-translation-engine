// //////////////////////////////////////////////////////////
// digest.rs
// Converted from digest.cpp
// Copyright (c) 2014,2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html
//

use std::env;
use std::fs::File;
use std::io::{self, Read, BufReader};

mod crc32 {
    pub struct CRC32;
    impl CRC32 {
        pub fn new() -> Self { CRC32 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "dummy_crc32_hash".to_string() }
    }
}

mod md5 {
    pub struct MD5;
    impl MD5 {
        pub fn new() -> Self { MD5 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "dummy_md5_hash".to_string() }
    }
}

mod sha1 {
    pub struct SHA1;
    impl SHA1 {
        pub fn new() -> Self { SHA1 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "dummy_sha1_hash".to_string() }
    }
}

mod sha256 {
    pub struct SHA256;
    impl SHA256 {
        pub fn new() -> Self { SHA256 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "dummy_sha256_hash".to_string() }
    }
}

mod keccak {
    pub enum KeccakMode {
        Keccak256,
    }

    pub struct Keccak;
    impl Keccak {
        pub fn new(_mode: KeccakMode) -> Self { Keccak }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "dummy_keccak_hash".to_string() }
    }
}

mod sha3 {
    pub enum Sha3Mode {
        Bits256,
    }

    pub struct SHA3;
    impl SHA3 {
        pub fn new(_mode: Sha3Mode) -> Self { SHA3 }
        pub fn add(&mut self, _data: &[u8]) {}
        pub fn get_hash(&self) -> String { "dummy_sha3_hash".to_string() }
    }
}

use crc32::CRC32;
use md5::MD5;
use sha1::SHA1;
use sha256::SHA256;
use keccak::Keccak;
use sha3::SHA3;

fn main() -> Result<(), i32> {
    // command line arguments
    let args: Vec<String> = env::args().collect();

    // syntax check
    if args.len() < 2 || args.len() > 3 {
        println!("./digest filename [--crc|--md5|--sha1|--sha256|--keccak|--sha3]");
        return Err(1);
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

    let mut digest_crc32 = CRC32::new();
    let mut digest_md5 = MD5::new();
    let mut digest_sha1 = SHA1::new();
    let mut digest_sha2 = SHA256::new();
    let mut digest_keccak = Keccak::new(keccak::KeccakMode::Keccak256);
    let mut digest_sha3 = SHA3::new(sha3::Sha3Mode::Bits256);

    // select input source: either file or standard-in
    let input: Box<dyn Read> = if filename == "-" {
        Box::new(io::stdin())
    } else {
        // open file
        let file = File::open(filename).map_err(|_| {
            eprintln!("Can't open '{}'", filename);
            2
        })?;
        Box::new(BufReader::new(file))
    };

    // each cycle processes about 1 MByte (divisible by 144 => improves Keccak/SHA3 performance)
    const BUFFER_SIZE: usize = 144 * 7 * 1024;
    let mut buffer = vec![0; BUFFER_SIZE];

    // process file
    let mut reader = input;
    loop {
        let num_bytes_read = match reader.read(&mut buffer) {
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

    Ok(())
}