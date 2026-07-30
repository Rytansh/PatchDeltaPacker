use serde_json;
use std::path::Path;
use std::{fs, io};

use crate::build::concurrency::worker_pool::WorkerPool;
use crate::build::{manifests, manifests::structs::Manifest};
use crate::constants::MANIFEST_RELATIVE_PATH;

pub async fn generate_manifest(
    root_directory_path: &Path,
    worker_pool: &WorkerPool,
) -> Result<Manifest, io::Error> //updates manifest if it exists, otherwise creates new manifest, returns manifest upon success
{
    let manifest_path = root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH));
    let manifest = manifests::builder::build_manifest(root_directory_path, worker_pool).await?;
    let json = serde_json::to_vec_pretty(&manifest)?;
    fs::write(manifest_path, json)?;

    Ok(manifest)
}
