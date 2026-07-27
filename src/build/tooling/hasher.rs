use sha2::{Digest, Sha256};
use xxhash_rust::xxh64::xxh64;

pub fn hash_sha256(contents: &[u8]) -> [u8; 32] {
    Sha256::digest(contents).into()
}

pub fn hash_xxh64(contents: &[u8], hash_seed: u64) -> u64 {
    xxh64(contents, hash_seed)
}
