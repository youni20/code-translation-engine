// //////////////////////////////////////////////////////////
// digest.rs
// Translation of digest.cpp into Rust

use std::env;
use std::fs::File;
use std::io::{self, Read, stdin};

// The module declarations for actual implementations have been commented out because the files are missing.
// To make this compile, either provide these modules, or comment them out like below:

// mod crc32;
// mod md5;
// mod sha1;
// mod sha256;
// mod keccak;
// mod sha3;

fn main() -> io::Result<()> {
    // syntax check
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        println!("./digest filename [--crc|--md5|--sha1|--sha256|--keccak|--sha3]");
        return Ok(());
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

    // Dummy structs with necessary methods for successful compilation.
    // Replace them with actual module instances with correct methods
    struct CRC32;
    impl CRC32 {
        fn new() -> Self { CRC32 }
        fn add(&mut self, _data: &[u8]) {}
        fn get_hash(&self) -> &'static str { "dummy_crc32_hash" }
    }

    struct MD5;
    impl MD5 {
        fn new() -> Self { MD5 }
        fn add(&mut self, _data: &[u8]) {}
        fn get_hash(&self) -> &'static str { "dummy_md5_hash" }
    }

    struct SHA1;
    impl SHA1 {
        fn new() -> Self { SHA1 }
        fn add(&mut self, _data: &[u8]) {}
        fn get_hash(&self) -> &'static str { "dummy_sha1_hash" }
    }

    struct SHA256;
    impl SHA256 {
        fn new() -> Self { SHA256 }
        fn add(&mut self, _data: &[u8]) {}
        fn get_hash(&self) -> &'static str { "dummy_sha256_hash" }
    }

    struct Keccak;
    impl Keccak {
        fn new(_type: KeccakType) -> Self { Keccak }
        fn add(&mut self, _data: &[u8]) {}
        fn get_hash(&self) -> &'static str { "dummy_keccak_hash" }
    }

    enum KeccakType {
        Keccak256,
    }

    struct SHA3;
    impl SHA3 {
        fn new(_type: SHA3Type) -> Self { SHA3 }
        fn add(&mut self, _data: &[u8]) {}
        fn get_hash(&self) -> &'static str { "dummy_sha3_hash" }
    }

    enum SHA3Type {
        Bits256,
    }

    let mut digest_crc32 = CRC32::new();
    let mut digest_md5 = MD5::new();
    let mut digest_sha1 = SHA1::new();
    let mut digest_sha2 = SHA256::new();
    let mut digest_keccak = Keccak::new(KeccakType::Keccak256);
    let mut digest_sha3 = SHA3::new(SHA3Type::Bits256);

    // select input source: either file or standard-in
    let input: Box<dyn Read> = if filename == "-" {
        Box::new(stdin())
    } else {
        let file = File::open(filename)?;
        Box::new(file)
    };

    // each cycle processes about 1 MByte (divisible by 144 => improves Keccak/SHA3 performance)
    const BUFFER_SIZE: usize = 144 * 7 * 1024;
    let mut buffer = vec![0; BUFFER_SIZE];

    // process file
    let mut input = input.take(u64::MAX); // replicate behavior of unlimited `read` in C++
    while let Ok(num_bytes_read) = input.read(&mut buffer) {
        if num_bytes_read == 0 { break; }

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