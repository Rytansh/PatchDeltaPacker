use std::path::PathBuf;

use crate::chunker::chunk_structs::ChunkMetadata;

use serde;

#[derive(Debug)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Manifest {
    pub manifest_ver : String,
    pub game_ver: String,
    pub chunk_size: usize,
    pub files : Vec<ManifestFile>
}

#[derive(Debug)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[derive(Clone)]
pub struct ManifestFile {
    pub file_hash: u64,
    pub file_path: PathBuf,
    pub file_size: usize,
    pub chunk_data: Vec<ChunkMetadata>
}