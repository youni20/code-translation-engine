use std::io::{self, Write};
use std::string::FromUtf8Error;

trait Digest {
    fn digest(&self, input: &[u8]) -> String;
}

#[derive(Default)]
struct SHA1;

impl Digest for SHA1 {
    fn digest(&self, input: &[u8]) -> String {
        // Simulate SHA1 digest calculation
        format!("SHA1({:?})", input)
    }
}

#[derive(Default)]
struct SHA256;

impl Digest for SHA256 {
    fn digest(&self, input: &[u8]) -> String {
        // Simulate SHA256 digest calculation
        format!("SHA256({:?})", input)
    }
}

#[derive(Default)]
struct CRC32;

impl Digest for CRC32 {
    fn digest(&self, input: &[u8]) -> String {
        // Simulate CRC32 digest calculation
        format!("CRC32({:?})", input)
    }
}

#[derive(Default)]
struct MD5;

impl Digest for MD5 {
    fn digest(&self, input: &[u8]) -> String {
        // Simulate MD5 digest calculation
        format!("MD5({:?})", input)
    }
}

#[derive(Default)]
struct HMAC;

impl Digest for HMAC {
    fn digest(&self, input: &[u8]) -> String {
        // Simulate HMAC digest calculation
        format!("HMAC({:?})", input)
    }
}

fn sha1_hash(input: &[u8]) -> String {
    let hasher: SHA1 = Default::default();
    hasher.digest(input)
}

fn sha256_hash(input: &[u8]) -> String {
    let hasher: SHA256 = Default::default();
    hasher.digest(input)
}

fn sha3_hash<S: Digest + Default>(size: usize) -> S {
    match size {
        256 => Default::default(),
        224 => Default::default(),
        512 => Default::default(),
        _ => panic!("Unsupported SHA3 size"),
    }
}

fn crc32_hash(input: &[u8]) -> String {
    let hasher: CRC32 = Default::default();
    hasher.digest(input)
}

fn md5_hash(input: &[u8]) -> String {
    let hasher: MD5 = Default::default();
    hasher.digest(input)
}

fn hmac_md5_hash(input: &[u8], _key: &[u8]) -> String {
    let hasher: HMAC = Default::default();
    hasher.digest(input)
}

fn hmac_sha1_hash(input: &[u8], _key: &[u8]) -> String {
    let hasher: HMAC = Default::default();
    hasher.digest(input)
}

fn hmac_sha256_hash(input: &[u8], _key: &[u8]) -> String {
    let hasher: HMAC = Default::default();
    hasher.digest(input)
}

#[derive(Debug)]
struct TestSet {
    input: &'static str,
    crc32b: &'static str,
    md5: &'static str,
    sha1: &'static str,
    sha256: &'static str,
    sha3_256: &'static str,
}

const NUM_TESTS: usize = 2;
const TESTSET: [TestSet; NUM_TESTS] = [
    TestSet { input: "cc", crc32b: "40d06116", md5: "a2e970f170961ce879190d64982c94ec", sha1: "a6f57425137e9aa54537f0b3f5364ce165aedb0a", sha256: "1dd8312636f6a0bf3d21fa2855e63072507453e93a5ced4301b364e91c9d87d6", sha3_256: "677035391cd3701293d385f037ba32796252bb7ce180b00b582dd9b20aaad7f0" },
    TestSet { input: "433c5303131624c0021d868a30825475e8d0bd3052a022180398f4ca4423b98214b6beaac21c8807a2c33f8c93bd42b092cc1b06cedf3224d5ed1ec29784444f22e08a55aa58542b524b02cd3d5d5f6907afe71c5d7462224a3f9d9e53e7e0846dcbb4ce", crc32b: "5c444498", md5: "9dc41264137166fe20aebb253ecce43e", sha1: "e9c3b6728e90f15a3703d1b9906e8f957ce0d4e5", sha256: "19acbb45e086963576fa1847f933f6ed78e777a4a27aca0609969362a72e3abf", sha3_256: "90e10b1ca8d352794d7dbd7bae410bef25f0ec7d080e053f48674237e33ea45f" },
];

fn check<HashMethod, InputFn>(input: &[u8], expected_result: &str, hasher: InputFn) -> usize
where
    HashMethod: Digest + Default,
    InputFn: Fn(&[u8]) -> String,
{
    let hash = hasher(&input);
    if hash == expected_result {
        0
    } else {
        writeln!(io::stderr(), "hash failed ! expected \"{}\" but library computed \"{}\"", expected_result, hash).unwrap();
        1
    }
}

fn check_sha3<HashMethod, F>(input: &[u8], expected_result: &str, hash_size: usize, hasher: F) -> usize
where
    HashMethod: Digest + Default,
    F: FnOnce(usize) -> HashMethod,
{
    let hash = hasher(hash_size).digest(input);
    if hash == expected_result {
        0
    } else {
        writeln!(io::stderr(), "hash/{} failed ! expected \"{}\" but library computed \"{}\"", hash_size, expected_result, hash).unwrap();
        1
    }
}

fn check_hmac<HashMethod, InputFn>(input: &[u8], key: &[u8], expected_result: &str, hasher: InputFn) -> usize
where
    HashMethod: Digest + Default,
    InputFn: Fn(&[u8], &[u8]) -> String,
{
    let hash = hasher(input, key);
    if hash == expected_result {
        0
    } else {
        writeln!(io::stderr(), "hmac hash failed ! expected \"{}\" but library computed \"{}\"", expected_result, hash).unwrap();
        1
    }
}

fn hex2bin(hex: &str) -> Result<Vec<u8>, FromUtf8Error> {
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>();
    Ok(bytes)
}

fn main() -> io::Result<()> {
    let mut errors = 0;

    let abc = "abc".as_bytes();
    let abc448bits = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let abc896bits = b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    let million = vec![b'a'; 1_000_000];

    println!("test SHA1 ...");
    errors += check::<SHA1, _>(b"", "da39a3ee5e6b4b0d3255bfef95601890afd80709", sha1_hash);
    errors += check::<SHA1, _>(abc, "a9993e364706816aba3e25717850c26c9cd0d89d", sha1_hash);
    errors += check::<SHA1, _>(abc448bits, "84983e441c3bd26ebaae4aa1f95129e5e54670f1", sha1_hash);
    errors += check::<SHA1, _>(abc896bits.as_slice(), "a49b2446a02c645bf419f995b67091253a04a259", sha1_hash);
    errors += check::<SHA1, _>(million.as_slice(), "34aa973cd4c4daa4f61eeb2bdbad27316534016f", sha1_hash);

    println!("test SHA2/256 ...");
    errors += check::<SHA256, _>(b"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", sha256_hash);
    errors += check::<SHA256, _>(abc, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", sha256_hash);
    errors += check::<SHA256, _>(abc448bits, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1", sha256_hash);
    errors += check::<SHA256, _>(abc896bits.as_slice(), "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1", sha256_hash);
    errors += check::<SHA256, _>(million.as_slice(), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0", sha256_hash);

    println!("test SHA3/256 ...");
    errors += check_sha3::<SHA1, _>(b"", "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a", 256, sha3_hash);
    errors += check_sha3::<SHA1, _>(abc, "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532", 256, sha3_hash);
    errors += check_sha3::<SHA1, _>(abc448bits, "41c0dba2a9d6240849100376a8235e2c82e1b9998a999e21db32dd97496d3376", 256, sha3_hash);
    errors += check_sha3::<SHA1, _>(abc896bits.as_slice(), "916f6061fe879741ca6469b43971dfdb28b1a32dc36cb3254e812be27aad1d18", 256, sha3_hash);
    errors += check_sha3::<SHA1, _>(million.as_slice(), "5c8875ae474a3634ba4fd55ec85bffd661f32aca75c6d699d0cdcb6c115891c1", 256, sha3_hash);

    println!("test SHA3/512 ...");
    let sha3_512_bug = hex2bin("13bd2811f6ed2b6f04ff3895aceed7bef8dcd45eb121791bc194a0f806206bffc3b9281c2b308b1a729ce008119dd3066e9378acdcc50a98a82e20738800b6cddbe5fe9694ad6d").unwrap();
    let result: SHA1 = Default::default(); 
    let hash = result.digest(&sha3_512_bug);
    if hash != "def4ab6cda8839729a03e000846604b17f03c5d5d7ec23c483670a13e11573c1e9347a63ec69a5abb21305f9382ecdaaabc6850f92840e86f88f4dabfcd93cc0" {
        writeln!(io::stderr(), "SHA3/512 bug present").unwrap();
        errors += 1;
    }

    println!("test SHA3/224 ...");
    let sha3_224 = "6b4e03423667dbb73b6e15454f0eb1abd4597f9a1b078e3f5b5a6bc7";
    if check_sha3::<SHA1, _>(b"", sha3_224, 224, sha3_hash) != 0 {
        writeln!(io::stderr(), "SHA3/224 bug present").unwrap();
        errors += 1;
    }
    println!("test Keccak/224 ...");
    let keccak224: HMAC = Default::default();
    let hash = keccak224.digest(b"");
    if hash != "f71837502ba8e10837bdd8d365adb85591895602fc552b48b7390abd" {
        writeln!(io::stderr(), "Keccak/224 bug present").unwrap();
        errors += 1;
    }

    println!("generic testsets (CRC32,MD5,SHA1,SHA256,SHA3) ...");
    for i in 0..NUM_TESTS {
        let test = &TESTSET[i];
        let input = hex2bin(test.input).unwrap();
        errors += check::<CRC32, _>(&input, test.crc32b, crc32_hash);
        errors += check::<MD5, _>(&input, test.md5, md5_hash);
        errors += check::<SHA1, _>(&input, test.sha1, sha1_hash);
        errors += check::<SHA256, _>(&input, test.sha256, sha256_hash);
        errors += check_sha3::<SHA1, _>(&input, test.sha3_256, 256, sha3_hash);
    }

    println!("test HMAC(MD5) ...");
    errors += check_hmac::<HMAC, _>("Hi There".as_bytes(), &hex2bin("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap(), "9294727a3638bb1c13f48ef8158bfc9d", hmac_md5_hash);
    errors += check_hmac::<HMAC, _>("Hi There".as_bytes(), &hex2bin("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap(), "b617318655057264e28bc0b6fb378c8ef146be00", hmac_sha1_hash);

    println!("test HMAC(SHA256) ...");
    errors += check_hmac::<HMAC, _>("Hi There".as_bytes(), &hex2bin("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap(), "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7", hmac_sha256_hash);

    if errors == 0 {
        println!("all tests ok");
    } else {
        println!("{} tests failed", errors);
    }

    Ok(())
}