use std::path::PathBuf;

use crate::chunker::chunk_structs::ChunkMetadata;

use serde;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub manifest_ver: String,
    pub game_ver: String,
    pub chunk_size: usize,
    pub files: Vec<ManifestFile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ManifestFile {
    pub file_hash: u64,
    pub file_path: PathBuf,
    pub file_size: usize,
    pub chunk_data: Vec<ChunkMetadata>,
}

pub struct ManifestJob {
    pub file_path: PathBuf,
    pub file_contents: Vec<u8>,
    pub job_index: usize,
    pub file_size: usize
}
pub struct ManifestJobResult {
    pub file: ManifestFile,
    pub job_index: usize,
}

pub struct ManifestBuildContext {
    pub root_directory: PathBuf,
}
