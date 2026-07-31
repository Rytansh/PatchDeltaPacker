use std::path::PathBuf;

use crate::build::manifests::structs::ManifestFile;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PatchPlan {
    pub old_ver: String,
    pub new_ver: String,
    pub added_files: Vec<ManifestFile>,
    pub deleted_files: Vec<ManifestFile>,
    pub modified_files: Vec<Modification>,
    pub chunk_size: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Modification {
    pub file_path: PathBuf,
    pub modified_chunks_indices: Vec<usize>,
    pub added_chunks_indices: Vec<usize>,
    pub deleted_chunks_indices: Vec<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PatchPackage {
    pub old_ver: String,
    pub new_ver: String,
    pub chunk_size: usize,

    pub added_files: Vec<AddedFile>,
    pub modified_files: Vec<ModifiedFile>,
    pub deleted_files: Vec<DeletedFile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddedFile {
    pub file_path: PathBuf,
    pub bytes_added: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ModifiedFile {
    pub file_path: PathBuf,
    pub target_file_size: usize,
    pub modified_chunks: Vec<ModifiedChunk>,
    pub added_chunks: Vec<ModifiedChunk>,
    pub deleted_chunks: Vec<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeletedFile {
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModifiedChunk {
    pub index: usize,
    pub bytes: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PatchHistory {
    pub latest_version: String,
    pub patches: Vec<PatchEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchEntry {
    pub from: String,
    pub to: String,
    pub file: String,
    pub size: u64,
    pub checksum: [u8; 32],
}
