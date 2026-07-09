use serde;

#[derive(Debug)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[derive(Clone)]
pub struct ChunkMetadata {
    pub hash : u64
}

pub struct Chunk {
    pub contents : Vec<u8>,
    pub hash : u64
}

pub struct ChunkedFile {
    pub chunks : Vec<Chunk>,
    pub hash: u64
}