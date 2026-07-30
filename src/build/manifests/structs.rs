use serde;
use std::path::PathBuf;

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
    pub file_size: u64,
    pub chunk_hashes: Vec<u64>,
}
