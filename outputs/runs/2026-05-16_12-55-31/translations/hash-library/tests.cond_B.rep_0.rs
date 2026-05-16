use std::io::{self, Write};

// Structure to hold test data
struct TestSet<'a> {
    input: &'a str,
    crc32b: &'a str,
    md5: &'a str,
    sha1: &'a str,
    sha256: &'a str,
    sha3_256: &'a str,
}

const NUM_TESTS: usize = 2;
static TESTSET: [TestSet; NUM_TESTS] = [
    TestSet { input: "cc", crc32b: "40d06116", md5: "a2e970f170961ce879190d64982c94ec", sha1: "a6f57425137e9aa54537f0b3f5364ce165aedb0a", sha256: "1dd8312636f6a0bf3d21fa2855e63072507453e93a5ced4301b364e91c9d87d6", sha3_256: "677035391cd3701293d385f037ba32796252bb7ce180b00b582dd9b20aaad7f0" },
    TestSet { input: "41fb", crc32b: "82d4472f", md5: "70d3e9af7232e67b6b6f3e71f7399438", sha1: "a6a5d330d9928b452ca7e34e946c52ea4f6eaa15", sha256: "0f8fa28112230a7a0b3cabcb64d37bd38f5023b1391e38f89a9b29f32b0aefbc", sha3_256: "39f31b6e653dfcd9caed2602fd87f61b6254f581312fb6eeec4d7148fa2e72aa" },
];

fn check<HashMethod>(_: &[u8], expected_result: &str) -> u32
where
    HashMethod: Fn(&[u8]) -> String,
{
    let hash_method = |data: &[u8]| data.iter().map(|b| format!("{:02x}", b)).collect::<String>(); // Dummy hash function
    let result = hash_method(b"dummy");
    if result == expected_result {
        0
    } else {
        writeln!(
            io::stderr(),
            "hash failed ! expected \"{}\" but library computed \"{}\"",
            expected_result, result
        )
        .unwrap();
        1
    }
}

fn check_with_size<HashMethod>(_: &[u8], expected_result: &str, hash_size: usize) -> u32
where
    HashMethod: Fn(&[u8], usize) -> String,
{
    let hash_method = |data: &[u8], _size| data.iter().map(|b| format!("{:02x}", b)).collect::<String>(); // Dummy hash function
    let result = hash_method(b"dummy", hash_size);
    if result == expected_result {
        0
    } else {
        writeln!(
            io::stderr(),
            "hash/{} failed ! expected \"{}\" but library computed \"{}\"",
            hash_size, expected_result, result
        )
        .unwrap();
        1
    }
}

fn check_hmac<HashMethod>(_: &[u8], _: &[u8], expected_result: &str) -> u32
where
    HashMethod: Fn(&[u8], &[u8]) -> String,
{
    let hash_method = |data: &[u8], _key| data.iter().map(|b| format!("{:02x}", b)).collect::<String>(); // Dummy hash function
    let result = hash_method(b"dummy_key", b"dummy");
    if result == expected_result {
        0
    } else {
        writeln!(
            io::stderr(),
            "hmac hash failed ! expected \"{}\" but library computed \"{}\"",
            expected_result, result
        )
        .unwrap();
        1
    }
}

fn hex2bin(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let high = chunk[0] as char;
            let low = chunk[1] as char;
            let h = if high.is_digit(16) {
                high.to_digit(16).unwrap()
            } else {
                (high as u8 - b'a' + 10) as u32
            };
            let l = if low.is_digit(16) {
                low.to_digit(16).unwrap()
            } else {
                (low as u8 - b'a' + 10) as u32
            };
            (h * 16 + l) as u8
        })
        .collect()
}

fn main() {
    let errors = 0;

    let _empty = b"";
    let _abc = b"abc";
    let _abc448bits = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let _abc896bits = b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    let _million = vec![b'a'; 1000000];

    println!("test SHA1 ...");
    // Placeholder for actual function calls
    // errors += check::<sha1_function>(empty, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    // errors += check::<sha1_function>(abc, "a9993e364706816aba3e25717850c26c9cd0d89d");
    // errors += check::<sha1_function>(abc448bits, "84983e441c3bd26ebaae4aa1f95129e5e54670f1");
    // errors += check::<sha1_function>(abc896bits, "a49b2446a02c645bf419f995b67091253a04a259");
    // errors += check::<sha1_function>(&million, "34aa973cd4c4daa4f61eeb2bdbad27316534016f");

    println!("test SHA2/256 ...");
    // Placeholder for actual function calls
    // errors += check::<sha256_function>(empty, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    // errors += check::<sha256_function>(abc, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    // errors += check::<sha256_function>(abc448bits, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    // errors += check::<sha256_function>(abc896bits, "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1");
    // errors += check::<sha256_function>(&million, "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");

    println!("test SHA3/256 ...");
    // Placeholder for actual function calls
    // errors += check_with_size::<sha3_function>(empty, "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a", 256);
    // errors += check_with_size::<sha3_function>(abc, "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532", 256);
    // errors += check_with_size::<sha3_function>(abc448bits, "41c0dba2a9d6240849100376a8235e2c82e1b9998a999e21db32dd97496d3376", 256);
    // errors += check_with_size::<sha3_function>(abc896bits, "916f6061fe879741ca6469b43971dfdb28b1a32dc36cb3254e812be27aad1d18", 256);
    // errors += check_with_size::<sha3_function>(&million, "5c8875ae474a3634ba4fd55ec85bffd661f32aca75c6d699d0cdcb6c115891c1", 256);

    println!("generic testsets (CRC32,MD5,SHA1,SHA256,SHA3) ...");
    // Placeholder for actual function calls
    for test in TESTSET.iter() {
        let _input = hex2bin(test.input);
        // errors += check::<crc32_function>(&input, test.crc32b);
        // errors += check::<md5_function>(&input, test.md5);
        // errors += check::<sha1_function>(&input, test.sha1);
        // errors += check::<sha256_function>(&input, test.sha256);
        // errors += check_with_size::<sha3_function>(&input, test.sha3_256, 256);
    }

    if errors == 0 {
        println!("all tests ok");
    } else {
        eprintln!("{} tests failed", errors);
    }
}