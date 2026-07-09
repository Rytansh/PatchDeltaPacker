use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::constants::MANIFEST_RELATIVE_PATH;
use crate::manifests::manifest_structs::Manifest;

use serde_json;

pub fn get_manifest(root_directory_path: &Path) -> Result<Manifest, io::Error>
{
    let manifest_path : PathBuf = root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH));
    let manifest_text = fs::read_to_string(manifest_path)?;
    let manifest : Manifest = serde_json::from_str(&manifest_text)?;

    Ok(manifest)
}