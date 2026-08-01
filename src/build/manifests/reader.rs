use serde_json;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::build::manifests::structs::Manifest;
use crate::constants::MANIFEST_RELATIVE_PATH;

pub fn get_manifest(root_directory_path: &Path) -> Result<Manifest, io::Error> {
    let manifest_path: PathBuf = root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH));
    let bytes = fs::read(manifest_path)?;
    let manifest = serde_json::from_slice(&bytes)?;

    Ok(manifest)
}
