use std::env;
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::process;
use std::hash::{Hasher, Hash};

// Placeholder structs and trait implementation
struct DummyHash {
    state: u64,
}

impl DummyHash {
    pub fn new() -> DummyHash {
        DummyHash { state: 0 }
    }
}

impl Hasher for DummyHash {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state = self.state.wrapping_add(*byte as u64);
        }
    }
}

impl Hash for DummyHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&[self.state as u8]);
    }
}

impl DummyHash {
    fn add(&mut self, data: &[u8]) {
        self.write(data);
    }

    fn get_hash(&self) -> String {
        format!("{:x}", self.finish())
    }
}

fn main() {
    // syntax check
    let args: Vec<String> = env::args().collect();
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

    let mut digest_crc32 = DummyHash::new();
    let mut digest_md5 = DummyHash::new();
    let mut digest_sha1 = DummyHash::new();
    let mut digest_sha2 = DummyHash::new();
    let mut digest_keccak = DummyHash::new();
    let mut digest_sha3 = DummyHash::new();

    // select input source: either file or standard-in
    let input: Box<dyn Read> = if filename == "-" {
        Box::new(io::stdin())
    } else {
        // open file
        let file = File::open(filename);
        let file = match file {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Can't open '{}'", filename);
                process::exit(2);
            }
        };
        Box::new(BufReader::new(file))
    };

    // each cycle processes about 1 MByte (divisible by 144 => improves Keccak/SHA3 performance)
    const BUFFER_SIZE: usize = 144 * 7 * 1024;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    // process file
    let mut input = input;
    while let Ok(bytes_read) = input.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        if compute_crc32 {
            digest_crc32.add(&buffer[..bytes_read]);
        }
        if compute_md5 {
            digest_md5.add(&buffer[..bytes_read]);
        }
        if compute_sha1 {
            digest_sha1.add(&buffer[..bytes_read]);
        }
        if compute_sha2 {
            digest_sha2.add(&buffer[..bytes_read]);
        }
        if compute_keccak {
            digest_keccak.add(&buffer[..bytes_read]);
        }
        if compute_sha3 {
            digest_sha3.add(&buffer[..bytes_read]);
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