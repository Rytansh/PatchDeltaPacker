use std::fs;
use std::io;
use std::path::Path;

use crate::build::chunker::chunk_structs::{Chunk, ChunkedFile};

use crate::build::tooling::hasher;

pub fn chunk_file(file_path: &Path, chunk_size_in_bytes: usize) -> Result<ChunkedFile, io::Error> {
    let file_contents = fs::read(file_path)?;
    Ok(chunk_contents(file_contents, chunk_size_in_bytes))
}

pub fn chunk_bytes(file_contents: Vec<u8>, chunk_size_in_bytes: usize) -> ChunkedFile {
    chunk_contents(file_contents, chunk_size_in_bytes)
}

fn chunk_contents(file_contents: Vec<u8>, chunk_size_in_bytes: usize) -> ChunkedFile {
    let file_hash = hasher::hash_xxh64(&file_contents, 1);

    let mut file_chunks = Vec::new();
    let mut contents = Vec::with_capacity(chunk_size_in_bytes);

    for byte in &file_contents {
        if contents.len() == contents.capacity() {
            file_chunks.push(create_chunk(contents));
            contents = Vec::with_capacity(chunk_size_in_bytes);
        }

        contents.push(*byte);
    }

    file_chunks.push(create_chunk(contents));

    ChunkedFile {
        chunks: file_chunks,
        hash: file_hash,
    }
}

fn create_chunk(chunk_contents: Vec<u8>) -> Chunk {
    let hash = hasher::hash_xxh64(&chunk_contents, 1);
    Chunk {
        contents: chunk_contents,
        hash,
    }
}
