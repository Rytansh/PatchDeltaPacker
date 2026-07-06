use std::fs;
use std::io;
use std::path::{Path};

use xxhash_rust::xxh64::xxh64;

#[derive(Debug)]
pub struct Chunk {
    pub contents : Vec<u8>,
    pub hash : u64
}

pub struct ChunkedFile {
    pub chunks : Vec<Chunk>,
    pub hash: u64
}

pub fn chunk_file(file_path : &Path, chunk_size_in_bytes : usize) -> Result<ChunkedFile, io::Error> //converts a filepath into a chunked file, containing chunks and a file hash
{
    let file_contents : Vec<u8> = fs::read(file_path)?; //converts the file into a stream of bytes (Vec<u8>)

    let file_hash = hash_contents(&file_contents, &1);

    let mut file_chunks : Vec<Chunk> = Vec::new();
    let mut contents = Vec::with_capacity(chunk_size_in_bytes);

    for byte in &file_contents
    {
        if contents.len() == contents.capacity()
        {
            let chunk = create_chunk(contents);
            file_chunks.push(chunk);
            contents = Vec::with_capacity(chunk_size_in_bytes);
        }
        contents.push(*byte);
    }
    let chunk = create_chunk(contents);
    file_chunks.push(chunk);

    let chunked_file = ChunkedFile {
        chunks: file_chunks,
        hash: file_hash
    };
    
    return Ok(chunked_file)
}


fn create_chunk(chunk_contents : Vec<u8>) -> Chunk
{
    let hash = hash_contents(&chunk_contents, &1);
    return Chunk
    {
        contents: chunk_contents,
        hash: hash
    }
}


fn hash_contents(contents : &Vec<u8>, hash_seed : &u64) -> u64
{
    return xxh64(contents, *hash_seed)
}
