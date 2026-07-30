pub struct Chunk {
    pub hash: u64,
}

pub struct ChunkedFile {
    pub chunks: Vec<Chunk>,
    pub hash: u64,
}
