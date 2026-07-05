use std::fs;
use std::io;
use crate::chunk_builder;
use crate::chunk_extractor::{ChunkMetadata, extract_chunk_data};
use crate::directory_scanner;
use crate::config_reader;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Manifest {
    manifest_ver : String,
    game_ver: String,
    chunk_size: usize,
    files : Vec<ManifestFile>
}

#[derive(Debug)]
pub struct ManifestFile {
    file_path: PathBuf,
    file_size: usize,
    chunk_data: Vec<ChunkMetadata>
}

const MANIFEST_VERSION: &str = "1.0.0";
const CHUNK_SIZE: usize = 64;

pub fn build_manifest(root_directory_path: &Path) -> Result<Manifest, io::Error>
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
    let all_files = directory_scanner::scan_directory(root_directory_path)?;
    let mut manifest_files = Vec::with_capacity(all_files.len());
    
    for file in all_files {
        let m_file = build_manifest_file(root_directory_path, file.as_path())?;
        manifest_files.push(m_file);
    }

    Ok(manifest_files)
}

fn build_manifest_file(root_directory_path: &Path, file_path : &Path) -> Result<ManifestFile, io::Error>
{
    let size = fs::metadata(file_path)?.len() as usize;
    let chunked_file = chunk_builder::chunk_file(file_path, CHUNK_SIZE)?;
    let data = extract_chunk_data(chunked_file);
    let path = PathBuf::from(file_path.strip_prefix(root_directory_path).unwrap());
    let manifest_file = ManifestFile
    {
        file_path: path,
        file_size: size,
        chunk_data: data
    };
    Ok(manifest_file)
}