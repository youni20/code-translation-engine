use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

// Assume that the modules `crc32`, `md5`, `sha1`, `sha256`, `keccak`, `sha3` exist in the correct paths
// below you would replace this with module import statements that work for your file structure such as:
// use crate::path_to_crc32::crc32;
// use crate::path_to_md5::md5;
// use crate::path_to_sha1::sha1;
// use crate::path_to_sha256::sha256;
// use crate::path_to_keccak::keccak;
// use crate::path_to_sha3::sha3;

// Since the required modules don't exist in the example the code cannot actually compile until they are given

fn main() {
    let args: Vec<String> = env::args().collect();

    // syntax check
    if args.len() < 2 || args.len() > 3 {
        eprintln!("./digest filename [--crc|--md5|--sha1|--sha256|--keccak|--sha3]");
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

    // Example placeholder hashes, replace with actual hash types from imported modules:
    let mut digest_crc32 = DummyCRC32::new();
    let mut digest_md5 = DummyMD5::new();
    let mut digest_sha1 = DummySHA1::new();
    let mut digest_sha2 = DummySHA256::new();
    let mut digest_keccak = DummyKeccak::new();
    let mut digest_sha3 = DummySHA3::new();

    // select input source: either file or standard-in
    let mut input: Box<dyn Read> = if filename == "-" {
        Box::new(io::stdin())
    } else {
        let file = File::open(filename).unwrap_or_else(|_| {
            eprintln!("Can't open '{}'", filename);
            process::exit(2);
        });
        Box::new(file)
    };

    // each cycle processes about 1 MByte (divisible by 144 => improves Keccak/SHA3 performance)
    const BUFFER_SIZE: usize = 144 * 7 * 1024;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    // process file
    loop {
        let num_bytes_read = input.read(&mut buffer).unwrap_or(0);
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

// Dummy types for compilation example - replace with actual implementations.
struct DummyCRC32;
impl DummyCRC32 {
    fn new() -> Self { DummyCRC32 }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> String { "dummy_crc32_hash".into() }
}

struct DummyMD5;
impl DummyMD5 {
    fn new() -> Self { DummyMD5 }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> String { "dummy_md5_hash".into() }
}

struct DummySHA1;
impl DummySHA1 {
    fn new() -> Self { DummySHA1 }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> String { "dummy_sha1_hash".into() }
}

struct DummySHA256;
impl DummySHA256 {
    fn new() -> Self { DummySHA256 }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> String { "dummy_sha256_hash".into() }
}

struct DummyKeccak;
impl DummyKeccak {
    fn new() -> Self { DummyKeccak }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> String { "dummy_keccak_hash".into() }
}

struct DummySHA3;
impl DummySHA3 {
    fn new() -> Self { DummySHA3 }
    fn add(&mut self, _data: &[u8]) {}
    fn get_hash(&self) -> String { "dummy_sha3_hash".into() }
}