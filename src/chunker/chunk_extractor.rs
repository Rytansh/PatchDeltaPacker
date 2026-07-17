use crate::chunker::chunk_structs::{Chunk, ChunkMetadata};

pub fn extract_chunk_data(chunked_file: Vec<Chunk>) -> Vec<ChunkMetadata> //converts a stream of chunks into streams of chunk data
{
    let mut chunk_data = Vec::with_capacity(chunked_file.len());

    for chunk in chunked_file {
        let metadata = ChunkMetadata { hash: chunk.hash };
        chunk_data.push(metadata);
    }
    return chunk_data;
}
