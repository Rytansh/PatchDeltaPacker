use serde;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ChunkMetadata {
    pub hash: u64,
}

pub struct Chunk {
    pub contents: Vec<u8>,
    pub hash: u64,
}

pub struct ChunkedFile {
    pub chunks: Vec<Chunk>,
    pub hash: u64,
}
