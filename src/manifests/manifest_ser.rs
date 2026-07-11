use std::fs;
use std::io;
use std::path::{Path};
use crate::manifests::manifest_gen::build_manifest;
use crate::manifests::manifest_structs::Manifest;
use crate::constants::{MANIFEST_RELATIVE_PATH};
use serde_json;

pub fn write_manifest(root_directory_path: &Path) -> Result<Manifest, io::Error> //updates manifest if it exists, otherwise creates new manifest, returns manifest upon success
{
    let manifest_path = root_directory_path.join(Path::new(MANIFEST_RELATIVE_PATH));
    let manifest = build_manifest(root_directory_path)?;
    let json = serde_json::to_vec_pretty(&manifest)?;
    fs::write(manifest_path, json)?;

    Ok(manifest)
}