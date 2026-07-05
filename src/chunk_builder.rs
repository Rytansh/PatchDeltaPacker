use std::fs;
use xxhash_rust::xxh64::xxh64;
use std::io;
use std::path::{Path};

#[derive(Debug)]
pub struct Chunk {
    pub contents : Vec<u8>,
    pub hash : u64
}

pub fn chunk_file(file_path : &Path, chunk_size_in_bytes : usize) -> Result<Vec<Chunk>, io::Error> //converts a filepath into chunks of bytes
{
    let file_contents : Vec<u8> = fs::read(file_path)?; //converts the file into a stream of bytes (Vec<u8>)

    let mut chunked_file : Vec<Chunk> = Vec::new();

    let mut contents = Vec::with_capacity(chunk_size_in_bytes);
    for byte in &file_contents
    {
        if contents.len() == contents.capacity()
        {
            let chunk = create_chunk(contents);
            chunked_file.push(chunk);
            contents = Vec::with_capacity(chunk_size_in_bytes);
        }
        contents.push(*byte);
    }
    let chunk = create_chunk(contents);
    chunked_file.push(chunk);
    
    return Ok(chunked_file)
}


fn create_chunk(chunk_contents : Vec<u8>) -> Chunk
{
    let hash = hash_chunk(&chunk_contents, &1);
    return Chunk
    {
        contents: chunk_contents,
        hash: hash
    }
}


fn hash_chunk(chunk_contents : &Vec<u8>, hash_seed : &u64) -> u64
{
    return xxh64(chunk_contents, *hash_seed)
}