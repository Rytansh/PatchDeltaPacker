use std::path::Path;
use std::{fs, io};

use crate::build::patcher::patch_package_gen::build_patch;
use crate::build::patcher::patch_structs::PatchPackage;
use crate::build::tooling::compressor;
use crate::constants::PATCH_PACKAGES_PATH;

use crate::build::concurrency::worker_pool::WorkerPool;
use crate::build::patcher::patch_history;
use bincode::config;

pub async fn generate_patch(
    old_patch_root: &Path,
    new_patch_root: &Path,
    worker_pool: &WorkerPool,
) -> Result<PatchPackage, io::Error> {
    let patch = build_patch(old_patch_root, new_patch_root, worker_pool).await?;
    let output_path = Path::new(PATCH_PACKAGES_PATH);
    let output_file = output_path.join(format!("{}_to_{}.pdp", &patch.old_ver, &patch.new_ver));

    println!("Writing...");

    let bytes = bincode::serde::encode_to_vec(&patch, config::standard()).unwrap();
    let compressed_patch = compressor::compress(&bytes)?;
    fs::write(output_file, &compressed_patch)?;
    patch_history::update(&patch, &compressed_patch, Path::new(PATCH_PACKAGES_PATH))?;
    println!("Done!");

    Ok(patch)
}
