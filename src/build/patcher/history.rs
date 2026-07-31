use crate::build::patcher::structs::{PatchEntry, PatchHistory, PatchPackage};
use crate::build::tooling;
use crate::constants::PATCH_HISTORY_RELATIVE_PATH;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::{fs, io};

pub fn update(
    patch: &PatchPackage,
    contents: &[u8],
    patch_directory: &Path,
) -> Result<(), io::Error> {
    let mut patch_history = match get_history(patch_directory) {
        Ok(patch_history) => patch_history,
        Err(err) if err.kind() == io::ErrorKind::NotFound => PatchHistory {
            latest_version: patch.new_ver.clone(),
            patches: Vec::new(),
        },
        Err(err) => return Err(err),
    };

    let checksum = tooling::hasher::hash_sha256(contents);

    if !patch_history.patches.iter().any(|p| p.to == patch.new_ver) {
        patch_history.latest_version.clone_from(&patch.new_ver);
    }

    match patch_history
        .patches
        .iter_mut()
        .find(|p| p.from == patch.old_ver && p.to == patch.new_ver)
    {
        Some(existing) => {
            existing.file = format!("{}_to_{}.pdp", &patch.old_ver, &patch.new_ver);
            existing.size = contents.len() as u64;
            existing.checksum = checksum;
        }

        None => {
            patch_history.patches.push(PatchEntry {
                from: patch.old_ver.clone(),
                to: patch.new_ver.clone(),
                file: format!("{}_to_{}.pdp", &patch.old_ver, &patch.new_ver),
                size: contents.len() as u64,
                checksum,
            });
        }
    }

    let json = serde_json::to_string_pretty(&patch_history).map_err(io::Error::other)?;
    fs::write(
        patch_directory.join(Path::new(PATCH_HISTORY_RELATIVE_PATH)),
        json,
    )?;
    Ok(())
}

fn get_history(patch_directory: &Path) -> Result<PatchHistory, io::Error> {
    let patch_history_path = patch_directory.join(Path::new(PATCH_HISTORY_RELATIVE_PATH));
    let history_text = fs::read(patch_history_path)?;
    let history = serde_json::from_slice(&history_text).map_err(io::Error::other)?;

    Ok(history)
}

pub fn get_latest_version(patch_directory: &Path) -> Result<String, io::Error> {
    let history = get_history(patch_directory)?;
    Ok(history.latest_version)
}

pub fn get_patch_chain(
    patch_directory: &Path,
    current: &str,
    target: &str,
) -> io::Result<Vec<PatchEntry>> {
    let history = get_history(patch_directory)?;

    let mut lookup = HashMap::new();

    for patch in history.patches {
        if lookup.insert(patch.from.clone(), patch).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Duplicate source version in patch history.",
            ));
        }
    }

    let mut current = current.to_string();
    let mut visited = HashSet::new();
    let mut chain = Vec::new();

    while current != target {
        if !visited.insert(current.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Cycle detected in patch history.",
            ));
        }

        let patch = lookup
            .get(&current)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No patch exists from version {current}"),
                )
            })?
            .clone();

        current.clone_from(&patch.to);
        chain.push(patch);
    }

    Ok(chain)
}

pub fn get_patch_entry(
    patch_directory: &Path,
    old_ver: &str,
    new_ver: &str,
) -> Result<PatchEntry, io::Error> {
    let history = get_history(patch_directory)?;
    history
        .patches
        .iter()
        .find(|p| p.from == old_ver && p.to == new_ver)
        .cloned()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
}
