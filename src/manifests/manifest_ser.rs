use crate::concurrency::worker_pool::WorkerPool;
use crate::constants::MANIFEST_RELATIVE_PATH;
use crate::manifests::manifest_gen;
use crate::manifests::manifest_structs::Manifest;
use serde_json;
use std::fs;
use std::io;
use std::path::Path;

pub async fn write_manifest(
    root_directory_path: &Path,
    worker_pool: &WorkerPool,
) -> Result<Manifest, io::Error> //updates manifest if it exists, otherwise creates new manifest, returns manifest upon success
{
    let manifest_path = root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH));
    let manifest = manifest_gen::build_manifest(root_directory_path, worker_pool).await?;
    let json = serde_json::to_vec_pretty(&manifest)?;
    fs::write(manifest_path, json)?;

    Ok(manifest)
}
