use crate::build::patcher::patch_structs::{PatchEntry, PatchHistory, PatchPackage};
use crate::build::tooling::hasher;
use crate::constants::{PATCH_HISTORY_RELATIVE_PATH, PATCH_PACKAGES_PATH};
use std::path::Path;
use std::{fs, io};

pub fn update(
    patch: &PatchPackage,
    contents: &[u8],
    root_directory: &Path,
) -> Result<(), io::Error> {
    let mut patch_history = match get_history(root_directory) {
        Ok(patch_history) => patch_history,
        Err(err) if err.kind() == io::ErrorKind::NotFound => PatchHistory {
            latest_version: patch.new_ver.clone(),
            patches: Vec::new(),
        },
        Err(err) => return Err(err),
    };

    if !patch_history.patches.iter().any(|p| p.to == patch.new_ver) {
        patch_history.latest_version = patch.new_ver.clone();
    }

    match patch_history
        .patches
        .iter_mut()
        .find(|p| p.from == patch.old_ver && p.to == patch.new_ver)
    {
        Some(existing) => {
            existing.file = format!("{}_to_{}.pdp", &patch.old_ver, &patch.new_ver);
            existing.size = contents.len() as u64;
            existing.checksum = hasher::hash_sha256(contents);
        }

        None => {
            patch_history.patches.push(PatchEntry {
                from: patch.old_ver.clone(),
                to: patch.new_ver.clone(),
                file: format!("{}_to_{}.pdp", &patch.old_ver, &patch.new_ver),
                size: contents.len() as u64,
                checksum: hasher::hash_sha256(contents),
            });
        }
    }

    let json = serde_json::to_string_pretty(&patch_history).map_err(io::Error::other)?;
    fs::write(
        Path::new(PATCH_PACKAGES_PATH).join(Path::new(PATCH_HISTORY_RELATIVE_PATH)),
        json,
    )?;
    Ok(())
}

pub fn get_history(root_directory: &Path) -> Result<PatchHistory, io::Error> {
    let patch_history_path =
        Path::new(PATCH_PACKAGES_PATH).join(Path::new(PATCH_HISTORY_RELATIVE_PATH));
    let history_text = fs::read_to_string(patch_history_path)?;
    let history = serde_json::from_str(&history_text).map_err(io::Error::other)?;

    Ok(history)
}

pub fn get_latest_version(root_directory: &Path) -> Result<String, io::Error> {
    let history = get_history(root_directory)?;
    Ok(history.latest_version)
}

pub fn get_patch_entry(
    root_directory: &Path,
    old_ver: &str,
    new_ver: &str,
) -> Result<PatchEntry, io::Error> {
    let mut history = get_history(root_directory)?;
    history
        .patches
        .iter()
        .find(|p| p.from == old_ver && p.to == new_ver)
        .cloned()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
}
