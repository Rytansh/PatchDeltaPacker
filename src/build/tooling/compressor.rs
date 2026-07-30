use std::io;

use crate::constants::COMPRESSION_LEVEL;

pub fn compress(bytes: &[u8]) -> Result<Vec<u8>, io::Error> {
    zstd::encode_all(bytes, COMPRESSION_LEVEL)
}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, io::Error> {
    zstd::decode_all(bytes)
}
