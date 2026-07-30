use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::build::concurrency::worker_pool::WorkerPool;
use crate::build::manifests::structs::{Manifest, ManifestFile};
use crate::build::{chunking, config, tooling};
use crate::constants::{CHUNK_SIZE, MANIFEST_RELATIVE_PATH, MANIFEST_VERSION};

pub async fn build_manifest(
    root_directory_path: &Path,
    worker_pool: &WorkerPool,
) -> Result<Manifest, io::Error> // only builds manifest in memory
{
    let manifest_files = collect_manifest_files(root_directory_path, worker_pool).await?;
    let game_version = config::reader::get_game_version(root_directory_path)?;

    let manifest = Manifest {
        manifest_ver: String::from(MANIFEST_VERSION),
        game_ver: game_version,
        chunk_size: CHUNK_SIZE,
        files: manifest_files,
    };

    Ok(manifest)
}

async fn collect_manifest_files(
    root_directory_path: &Path,
    worker_pool: &WorkerPool,
) -> Result<Vec<ManifestFile>, io::Error> {
    let all_files: Vec<PathBuf> = tooling::directory_scanner::scan_directory(root_directory_path)?;
    let manifest_path = root_directory_path.join(MANIFEST_RELATIVE_PATH);
    let mut handles = Vec::new();

    for file_path in all_files {
        if file_path == manifest_path {
            continue;
        }

        let root_directory = root_directory_path.to_path_buf();
        let handle = worker_pool.execute(move || build_manifest_file(&root_directory, &file_path));
        handles.push(handle);
    }

    let mut manifest_files = Vec::with_capacity(handles.len());

    for handle in handles {
        let manifest = handle.wait().await?;
        manifest_files.push(manifest);
    }

    Ok(manifest_files)
}

fn build_manifest_file(
    root_directory_path: &Path,
    file_path: &Path,
) -> Result<ManifestFile, io::Error> {
    let chunked_file = chunking::file_chunker::chunk_file(file_path, CHUNK_SIZE)?;
    let metadata = fs::metadata(file_path)?;
    let size = metadata.len();
    let manifest_file = ManifestFile {
        file_hash: chunked_file.hash,
        file_path: PathBuf::from(file_path.strip_prefix(root_directory_path).unwrap()),
        file_size: size,
        chunk_hashes: chunking::file_chunker::collect_chunk_hashes(chunked_file),
    };
    Ok(manifest_file)
}
