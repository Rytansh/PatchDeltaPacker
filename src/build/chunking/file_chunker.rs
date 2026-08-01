use std::fs::File;
use std::path::Path;
use std::{io, io::Read};

use crate::build::{
    chunking::structs::{Chunk, ChunkedFile},
    tooling,
};
use crate::constants::HASH_SEED;

pub fn chunk_file(file_path: &Path, chunk_size: usize) -> Result<ChunkedFile, io::Error> {
    let mut file = File::open(file_path)?;
    let mut file_hasher = tooling::hasher::create_xxh64(HASH_SEED);

    let mut buffer = vec![0u8; chunk_size];
    let mut chunks = Vec::new();

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let slice = &buffer[..bytes_read];
        file_hasher.update(slice);
        let chunk_hash = tooling::hasher::hash_xxh64(slice);

        chunks.push(Chunk { hash: chunk_hash });
    }

    Ok(ChunkedFile {
        chunks,
        hash: file_hasher.digest(),
    })
}

pub fn collect_chunk_hashes(file: ChunkedFile) -> Vec<u64> {
    file.chunks.into_iter().map(|chunk| chunk.hash).collect()
}
