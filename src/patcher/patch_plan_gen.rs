use std::io;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::manifests::manifest_structs::{Manifest, ManifestFile};


#[derive(Debug)]
pub struct PatchPlan {
    old_ver: String,
    new_ver: String,
    added_files: Vec<ManifestFile>,
    deleted_files: Vec<ManifestFile>,
    modified_files: Vec<ModifiedFile>,
    chunk_size: usize
}

#[derive(Debug)]
pub struct ModifiedFile {
    file_path: PathBuf,
    modified_chunk_indices: Vec<usize>,
    added_chunks_indices: Vec<usize>,
    deleted_chunks_indices: Vec<usize>
}

pub fn create_patch_plan(old_manifest : &Manifest, new_manifest: &Manifest) -> Result<PatchPlan, io::Error>
{
    if old_manifest.chunk_size != new_manifest.chunk_size {
        return Err(io::Error::new(io::ErrorKind::InvalidData,"Chunk sizes differ."));
    }
    let old_version = old_manifest.game_ver.clone();
    let new_version = new_manifest.game_ver.clone();
    let mut old_file_lookup = HashMap::with_capacity(old_manifest.files.len());
    let mut new_file_lookup = HashMap::with_capacity(new_manifest.files.len());
    let mut new_modified_files = Vec::with_capacity(old_manifest.files.len());
    let mut new_added_files = Vec::with_capacity(new_manifest.files.len());
    let mut new_deleted_files = Vec::with_capacity(old_manifest.files.len());

    for file in &old_manifest.files
    {
        old_file_lookup.insert(&file.file_path,file);
    }
    for file in &new_manifest.files
    {
        new_file_lookup.insert(&file.file_path,file);
        match old_file_lookup.get(&file.file_path) {
            Some(existing_file) => {
                if file.file_hash == existing_file.file_hash {continue;}
                let modified_file = compare_modifications(&file, &existing_file); 
                new_modified_files.push(modified_file);
            }
            None => {
                new_added_files.push(file.clone());
            }
        }
    }
    for file in &old_manifest.files
    {
        match new_file_lookup.get(&file.file_path) {
            Some(_) => continue,
            None => {
                new_deleted_files.push(file.clone());
            }
        }
    }

    let patch_plan = PatchPlan {
        old_ver: old_version,
        new_ver: new_version,
        added_files: new_added_files,
        deleted_files: new_deleted_files,
        modified_files: new_modified_files,
        chunk_size: new_manifest.chunk_size
    };

    Ok(patch_plan)
}

fn compare_modifications(old_file: &ManifestFile, new_file: &ManifestFile) -> ModifiedFile
{
    let mut index = 0;
    let mut modified_indices = Vec::with_capacity(new_file.chunk_data.len());
    let mut deleted_indices = Vec::with_capacity(old_file.chunk_data.len());
    let mut added_indices = Vec::new();
    while index < new_file.chunk_data.len()
    {
        if index > old_file.chunk_data.len() - 1 {break;}
        if old_file.chunk_data[index].hash != new_file.chunk_data[index].hash {
            modified_indices.push(index);
        }
        index += 1;
    }

    if old_file.chunk_data.len() > new_file.chunk_data.len()
    {
        let num_deleted_chunks = old_file.chunk_data.len() - new_file.chunk_data.len();
        for i in 0..num_deleted_chunks {
            deleted_indices.push(index + i);
        }
    }
    else if new_file.chunk_data.len() > old_file.chunk_data.len()
    {
        let num_added_chunks = new_file.chunk_data.len() - old_file.chunk_data.len();
        for i in 0..num_added_chunks {
            added_indices.push(index + i);
        }
    }

    let modified_results = ModifiedFile {
        file_path: new_file.file_path.clone(),
        modified_chunk_indices: modified_indices,
        added_chunks_indices: added_indices,
        deleted_chunks_indices: deleted_indices
    };

    modified_results
}
