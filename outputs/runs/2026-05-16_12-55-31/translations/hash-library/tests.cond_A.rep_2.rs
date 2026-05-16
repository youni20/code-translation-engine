// //////////////////////////////////////////////////////////
// tests.rs
// Copyright (c) 2015 Stephan Brumme. All rights reserved.
// see http://create.stephan-brumme.com/disclaimer.html

// simple test suite for hash-library

// Assuming that these traits are defined in their respective modules:
trait HashFunction {
    fn hash(input: &[u8]) -> String;
}

fn hmac<HashMethod: HashFunction>(_input: &[u8], _key: &[u8]) -> String {
    // Dummy implementation
    "hash".to_string() // This needs to be replaced with the actual HMAC calculation
}

struct TestSet {
    input: &'static str,
    crc32b: &'static str,
    md5: &'static str,
    sha1: &'static str,
    sha256: &'static str,
    sha3_256: &'static str,
}

const NUM_TESTS: usize = 3; // Reduced for demonstration purposes
const TESTSET: [TestSet; NUM_TESTS] = [
    TestSet {
        input: "cc",
        crc32b: "40d06116",
        md5: "a2e970f170961ce879190d64982c94ec",
        sha1: "a6f57425137e9aa54537f0b3f5364ce165aedb0a",
        sha256: "1dd8312636f6a0bf3d21fa2855e63072507453e93a5ced4301b364e91c9d87d6",
        sha3_256: "677035391cd3701293d385f037ba32796252bb7ce180b00b582dd9b20aaad7f0",
    },
    TestSet {
        input: "41fb",
        crc32b: "82d4472f",
        md5: "70d3e9af7232e67b6b6f3e71f7399438",
        sha1: "a6a5d330d9928b452ca7e34e946c52ea4f6eaa15",
        sha256: "0f8fa28112230a7a0b3cabcb64d37bd38f5023b1391e38f89a9b29f32b0aefbc",
        sha3_256: "39f31b6e653dfcd9caed2602fd87f61b6254f581312fb6eeec4d7148fa2e72aa",
    },
    TestSet {
        input: "1f877c",
        crc32b: "c54a0ec4",
        md5: "f7ac37ba79246d6a36b49c0993791110",
        sha1: "d17212f7dbfa31d10b68d480bc91cd3e1596be86",
        sha256: "ab0213910396c8d94cbf3b6c97de1fc97fc55cea4b6b6cc25ea4b71e7e7bc28c",
        sha3_256: "bc22345e4bd3f792a341cf18ac0789f1c9c966712a501b19d1b6632ccd408ec5",
    },
    // ... (add remaining test vectors here)
];

fn hex2bin(hex: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(hex.len() / 2);
    let mut chars = hex.chars();
    while let (Some(high), Some(low)) = (chars.next(), chars.next()) {
        let high_digit = high.to_digit(16).unwrap() as u8;
        let low_digit = low.to_digit(16).unwrap() as u8;
        result.push(high_digit * 16 + low_digit);
    }
    result
}

fn check<HashMethod>(input: &[u8], expected: &str) -> u32
where
    HashMethod: HashFunction,
{
    let hash = HashMethod::hash(input);
    if hash == expected {
        0
    } else {
        eprintln!("hash failed! expected \"{}\" but library computed \"{}\"", expected, hash);
        1
    }
}

fn check_hmac<HashMethod>(input: &[u8], key: &[u8], expected: &str) -> u32
where
    HashMethod: HashFunction,
{
    let hash = hmac::<HashMethod>(input, key);
    if hash == expected {
        0
    } else {
        eprintln!("hmac hash failed! expected \"{}\" but library computed \"{}\"", expected, hash);
        1
    }
}

fn main() -> Result<(), String> {
    let mut errors = 0;

    let empty = Vec::from("");
    let abc = Vec::from("abc");
    let abc448bits = Vec::from("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
    let abc896bits = Vec::from(
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
    );
    let million = vec!['a' as u8; 1000000];

    // Insert compatibility or mock checks until actual crates are defined elsewhere
    {
        struct SHA1;
        struct SHA256;
        struct SHA3;
        struct CRC32;
        struct MD5;

        impl HashFunction for SHA1 {
            fn hash(_input: &[u8]) -> String {
                "dummy_sha1_hash".to_string()
            }
        }

        impl HashFunction for SHA256 {
            fn hash(_input: &[u8]) -> String {
                "dummy_sha256_hash".to_string()
            }
        }

        impl HashFunction for SHA3 {
            fn hash(_input: &[u8]) -> String {
                "dummy_sha3_hash".to_string()
            }
        }

        impl HashFunction for CRC32 {
            fn hash(_input: &[u8]) -> String {
                "dummy_crc32_hash".to_string()
            }
        }

        impl HashFunction for MD5 {
            fn hash(_input: &[u8]) -> String {
                "dummy_md5_hash".to_string()
            }
        }

        println!("test SHA1 ...");
        errors += check::<SHA1>(&empty, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        errors += check::<SHA1>(&abc, "a9993e364706816aba3e25717850c26c9cd0d89d");
        errors += check::<SHA1>(&abc448bits, "84983e441c3bd26ebaae4aa1f95129e5e54670f1");
        errors += check::<SHA1>(&abc896bits, "a49b2446a02c645bf419f995b67091253a04a259");
        errors += check::<SHA1>(&million, "34aa973cd4c4daa4f61eeb2bdbad27316534016f");

        println!("test SHA2/256 ...");
        errors += check::<SHA256>(&empty, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        errors += check::<SHA256>(&abc, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        errors += check::<SHA256>(&abc448bits, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
        errors += check::<SHA256>(&abc896bits, "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1");
        errors += check::<SHA256>(&million, "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");

        println!("test SHA3/256 ...");
        errors += check::<SHA3>(&empty, "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a");
        errors += check::<SHA3>(&abc, "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532");
        errors += check::<SHA3>(&abc448bits, "41c0dba2a9d6240849100376a8235e2c82e1b9998a999e21db32dd97496d3376");
        errors += check::<SHA3>(&abc896bits, "916f6061fe879741ca6469b43971dfdb28b1a32dc36cb3254e812be27aad1d18");
        errors += check::<SHA3>(&million, "5c8875ae474a3634ba4fd55ec85bffd661f32aca75c6d699d0cdcb6c115891c1");

        for i in 0..NUM_TESTS {
            errors += check::<CRC32>(&hex2bin(TESTSET[i].input), TESTSET[i].crc32b);
            errors += check::<MD5>(&hex2bin(TESTSET[i].input), TESTSET[i].md5);
            errors += check::<SHA1>(&hex2bin(TESTSET[i].input), TESTSET[i].sha1);
            errors += check::<SHA256>(&hex2bin(TESTSET[i].input), TESTSET[i].sha256);
            errors += check::<SHA3>(&hex2bin(TESTSET[i].input), TESTSET[i].sha3_256);
        }

        println!("test HMAC(MD5) ...");
        errors += check_hmac::<MD5>(&Vec::from("Hi There"), &hex2bin("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"), "9294727a3638bb1c13f48ef8158bfc9d");
        errors += check_hmac::<SHA1>(&Vec::from("Hi There"), &hex2bin("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"), "b617318655057264e28bc0b6fb378c8ef146be00");
        errors += check_hmac::<MD5>(&Vec::from("what do ya want for nothing?"), &Vec::from("Jefe"), "750c783e6ab0b503eaa86e310a5db738");
        errors += check_hmac::<SHA1>(&Vec::from("what do ya want for nothing?"), &Vec::from("Jefe"), "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79");
    }

    // Summing up the results
    if errors == 0 {
        println!("all tests ok");
        Ok(())
    } else {
        Err(format!("{} tests failed", errors))
    }
}