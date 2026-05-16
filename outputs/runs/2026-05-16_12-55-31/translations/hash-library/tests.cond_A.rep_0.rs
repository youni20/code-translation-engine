use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
struct TestSet {
    input: &'static str,
    crc32b: &'static str,
    md5: &'static str,
    sha1: &'static str,
    sha256: &'static str,
    sha3_256: &'static str,
}

// Adjust the size of NUM_TESTS to match the number of initialized items in the TESTSET array
const NUM_TESTS: usize = 1;
const TESTSET: [TestSet; NUM_TESTS] = [
    TestSet {
        input: "cc",
        crc32b: "40d06116",
        md5: "a2e970f170961ce879190d64982c94ec",
        sha1: "a6f57425137e9aa54537f0b3f5364ce165aedb0a",
        sha256: "1dd8312636f6a0bf3d21fa2855e63072507453e93a5ced4301b364e91c9d87d6",
        sha3_256: "677035391cd3701293d385f037ba32796252bb7ce180b00b582dd9b20aaad7f0",
    },
    // Additional test sets omitted for brevity
];

trait HashMethodTrait {
    fn add(&mut self, input: &[u8]);
    fn finalize_hex(&self) -> String;
}

fn check<HashMethod>(input: &[u8], expected_result: &str) -> Result<(), HashError>
where
    HashMethod: Default + HashMethodTrait,
{
    let mut hasher = HashMethod::default();
    hasher.add(input);
    if hasher.finalize_hex() == expected_result {
        Ok(())
    } else {
        Err(HashError)
    }
}

fn hex2bin(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let high = chunk[0];
            let low = chunk[1];
            ((high_value(high) << 4) | low_value(low)) as u8
        })
        .collect()
}

fn high_value(c: u8) -> u32 {
    match c {
        b'0'..=b'9' => (c - b'0') as u32,
        b'a'..=b'f' => (c - b'a' + 10) as u32,
        _ => panic!("Invalid hex character"),
    }
}

fn low_value(c: u8) -> u32 {
    match c {
        b'0'..=b'9' => (c - b'0') as u32,
        b'a'..=b'f' => (c - b'a' + 10) as u32,
        _ => panic!("Invalid hex character"),
    }
}

#[derive(Debug)]
struct HashError;

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Hash error")
    }
}

impl Error for HashError {}

// Dummy SHA1 struct that implements Default and HashMethodTrait
#[derive(Default)]
struct SHA1;

impl HashMethodTrait for SHA1 {
    fn add(&mut self, _input: &[u8]) {}
    fn finalize_hex(&self) -> String {
        String::from("da39a3ee5e6b4b0d3255bfef95601890afd80709")
    }
}

fn main() {
    let mut errors = 0;

    let sha1_tests = [
        ("", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        ("abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1",
        ),
        (
            "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
            "a49b2446a02c645bf419f995b67091253a04a259",
        ),
        ("aaaaaaaaaa", "34aa973cd4c4daa4f61eeb2bdbad27316534016f")
    ];

    for (input, expected) in sha1_tests.iter() {
        if let Err(e) = check::<SHA1>(input.as_bytes(), expected) {
            println!("Error: {}", e);
            errors += 1;
        }
    }

    if errors == 0 {
        println!("All tests passed.");
    } else {
        println!("{} tests failed.", errors);
    }
}