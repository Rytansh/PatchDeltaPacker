use std::fs;
use std::io;

use crate::chunker::chunk_builder;
use crate::chunker::chunk_extractor::extract_chunk_data;
use crate::concurrency::worker_pool::WorkerPool;
use crate::config::config_reader;
use crate::constants::{CHUNK_SIZE, MANIFEST_RELATIVE_PATH, MANIFEST_VERSION};
use crate::manifests::manifest_structs::{Manifest, ManifestFile};
use crate::tooling::directory_scanner;
use std::path::{Path, PathBuf};

pub async fn build_manifest(
    root_directory_path: &Path,
    worker_pool: &WorkerPool,
) -> Result<Manifest, io::Error> // only builds manifest in memory
{
    let manifest_files = collect_manifest_files(root_directory_path, worker_pool).await?;
    let game_version = config_reader::get_game_version(root_directory_path)?;

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
    let all_files: Vec<PathBuf> = directory_scanner::scan_directory(root_directory_path)?;
    let root_directory = root_directory_path.to_path_buf();
    let mut handles = Vec::new();

    for (index, filepath) in all_files.into_iter().enumerate() {
        if filepath.as_path() == root_directory.join(Path::new(MANIFEST_RELATIVE_PATH)) {
            continue;
        }
        let root_directory = root_directory.clone();
        let display_path = filepath.clone();

        let handle = worker_pool.execute(move || {
            println!(
                "[{:?}] Processing {}",
                std::thread::current().id(),
                display_path.display()
            );

            build_manifest_file(&root_directory, &filepath)
        });
        handles.push(handle);
    }

    let mut manifest_files = Vec::new();

    for handle in handles {
        let manifest = handle.wait().await?;
        manifest_files.push(manifest);
    }

    Ok(manifest_files)
}

pub fn build_manifest_file(
    root_directory_path: &Path,
    file_path: &Path,
) -> Result<ManifestFile, io::Error> {
    let size = usize::try_from(fs::metadata(file_path)?.len()).unwrap();
    let chunked_file = chunk_builder::chunk_file(file_path, CHUNK_SIZE)?;
    let data = extract_chunk_data(chunked_file.chunks);
    let path = PathBuf::from(file_path.strip_prefix(root_directory_path).unwrap());
    let manifest_file = ManifestFile {
        file_hash: chunked_file.hash,
        file_path: path,
        file_size: size,
        chunk_data: data,
    };
    Ok(manifest_file)
}
