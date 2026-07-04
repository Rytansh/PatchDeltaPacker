use std::fs;
use std::io;
use crate::chunk_builder;
use crate::chunk_extractor::{ChunkMetadata, extract_chunk_data};



#[derive(Debug)]
pub struct Manifest {
    manifest_ver : String,
    game_ver: String,
    chunk_size: usize,
    files : Vec<ManifestFile>
}

#[derive(Debug)]
pub struct ManifestFile {
    file_path: String,
    file_size: usize,
    chunk_data: Vec<ChunkMetadata>
}

pub fn build_manifest_file(filepath : &str) -> Result<ManifestFile, io::Error>
{
    let size = fs::metadata(filepath)?.len() as usize;
    let path = String::from(filepath);
    let chunked_file = chunk_builder::chunk_file(filepath, 32)?;
    let data =extract_chunk_data(chunked_file);
    let manifest_file = ManifestFile
    {
        file_path: path,
        file_size: size,
        chunk_data: data
    };
    Ok(manifest_file)
}