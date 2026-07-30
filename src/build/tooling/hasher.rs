use sha2::{Digest, Sha256};
use xxhash_rust::xxh64::{xxh64, Xxh64};

use crate::constants::HASH_SEED;

pub fn hash_sha256(contents: &[u8]) -> [u8; 32] {
    Sha256::digest(contents).into()
}

pub fn create_sha256() -> Sha256 {
    Sha256::new()
}

pub fn create_xxh64(seed: u64) -> Xxh64 {
    Xxh64::new(seed)
}

pub fn hash_xxh64(contents: &[u8]) -> u64 {
    xxh64(contents, HASH_SEED)
}
