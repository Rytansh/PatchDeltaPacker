use std::fs;
use std::io;

use std::path::{Path, PathBuf};
use crate::chunker::chunk_builder;
use crate::chunker::chunk_extractor::{ChunkMetadata, extract_chunk_data};
use crate::tooling::directory_scanner;
use crate::config::config_reader;
use crate::constants::{MANIFEST_RELATIVE_PATH, MANIFEST_VERSION, CHUNK_SIZE};

use xxhash_rust::xxh64::xxh64;
use serde_json;
use serde;


#[derive(Debug)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Manifest {
    manifest_ver : String,
    game_ver: String,
    chunk_size: usize,
    files : Vec<ManifestFile>
}

#[derive(Debug)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct ManifestFile {
    file_hash: u64,
    file_path: PathBuf,
    file_size: usize,
    chunk_data: Vec<ChunkMetadata>
}


pub fn write_manifest(root_directory_path: &Path) -> Result<Manifest, io::Error> //updates manifest if it exists, otherwise creates new manifest, returns manifest upon success
{
    let manifest_path = root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH));
    let manifest = build_manifest(root_directory_path)?;
    let json = serde_json::to_vec_pretty(&manifest)?;
    fs::write(manifest_path, json)?;

    Ok(manifest)
}

pub fn build_manifest(root_directory_path: &Path) -> Result<Manifest, io::Error> // only builds manifest in memory
{
    let manifest_files = collect_manifest_files(root_directory_path)?;
    let game_version = config_reader::get_game_version(root_directory_path)?;
    
    let manifest = Manifest {
        manifest_ver: String::from(MANIFEST_VERSION),
        game_ver: game_version,
        chunk_size: CHUNK_SIZE,
        files: manifest_files
    };

    Ok(manifest)
}


fn collect_manifest_files(root_directory_path: &Path) -> Result<Vec<ManifestFile>, io::Error>
{
    let all_files : Vec<PathBuf> = directory_scanner::scan_directory(root_directory_path)?;
    let mut manifest_files : Vec<ManifestFile> = Vec::with_capacity(all_files.len());
    
    for file in all_files {
        if file.as_path() == root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH)) {continue;}
        let m_file = build_manifest_file(root_directory_path, file.as_path())?;
        manifest_files.push(m_file);
    }

    Ok(manifest_files)
}

fn build_manifest_file(root_directory_path: &Path, file_path : &Path) -> Result<ManifestFile, io::Error>
{
    let size = fs::metadata(file_path)?.len() as usize;
    let chunked_file = chunk_builder::chunk_file(file_path, CHUNK_SIZE)?;
    let data = extract_chunk_data(chunked_file.chunks);
    let path = PathBuf::from(file_path.strip_prefix(root_directory_path).unwrap());
    let manifest_file = ManifestFile
    {
        file_hash: chunked_file.hash,
        file_path: path,
        file_size: size,
        chunk_data: data
    };
    Ok(manifest_file)
}