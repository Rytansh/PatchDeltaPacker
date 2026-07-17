use std::fs;
use std::io;
use std::path::Path;

use crate::chunker::chunk_structs::{Chunk, ChunkedFile};

use xxhash_rust::xxh64::xxh64;

pub fn chunk_file(file_path: &Path, chunk_size_in_bytes: usize,) -> Result<ChunkedFile, io::Error> 
{
    let file_contents = fs::read(file_path)?;
    Ok(chunk_contents(file_contents, chunk_size_in_bytes))
}

pub fn chunk_bytes(file_contents: Vec<u8>, chunk_size_in_bytes: usize) -> ChunkedFile 
{
    chunk_contents(file_contents, chunk_size_in_bytes)
}

fn chunk_contents(file_contents: Vec<u8>, chunk_size_in_bytes: usize) -> ChunkedFile {

    let file_hash = hash_contents(&file_contents, 1);

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

fn create_chunk(chunk_contents: Vec<u8> ) -> Chunk {
    let hash = hash_contents(&chunk_contents, 1);
    Chunk {
        contents: chunk_contents,
        hash,
    }
}

fn hash_contents(contents: &Vec<u8>, hash_seed: u64) -> u64 {
    xxh64(contents, hash_seed)
}
