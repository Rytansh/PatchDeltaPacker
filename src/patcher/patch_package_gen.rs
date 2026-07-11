use std::fs;
use std::io;
use std::path::Path;

use crate::patcher::patch_plan_gen::create_patch_plan;
use crate::patcher::patch_structs::{PatchPackage, AddedFile, ModifiedFile, DeletedFile, ModifiedChunk, PatchPlan};
use crate::manifests::manifest_ser::write_manifest;


pub fn build_patch(old_patch_root : &Path, new_patch_root : &Path) -> Result<PatchPackage, io::Error>
{
    let old_manifest = write_manifest(old_patch_root)?;
    let new_manifest = write_manifest(new_patch_root)?;
    let plan = create_patch_plan(&old_manifest, &new_manifest)?;
    let package = create_patch_package(&plan, new_patch_root)?;

    Ok(package)
}

pub fn create_patch_package(plan: &PatchPlan, new_patch_root : &Path) -> Result<PatchPackage, io::Error>
{
    let mut added_files : Vec<AddedFile> = Vec::with_capacity(plan.added_files.len());
    let mut deleted_files : Vec<DeletedFile> = Vec::with_capacity(plan.deleted_files.len());
    let mut modified_files: Vec<ModifiedFile> = Vec::with_capacity(plan.modified_files.len());
    let chunk_size = plan.chunk_size;

    for file in &plan.added_files 
    {
        let bytes : Vec<u8> = fs::read(new_patch_root.join(&file.file_path))?;
        let addition = AddedFile {
            file_path: file.file_path.clone(),
            bytes_added: bytes
        };
        added_files.push(addition);
    }

    for file in &plan.deleted_files
    {
        let deletion = DeletedFile {
            file_path: file.file_path.clone()
        };
        deleted_files.push(deletion);
    }

    for modification in &plan.modified_files
    {
        let bytes : Vec<u8> = fs::read(new_patch_root.join(&modification.file_path))?;
        let mut additions : Vec<ModifiedChunk> = Vec::with_capacity(modification.added_chunks_indices.len());
        let mut modifications : Vec<ModifiedChunk> = Vec::with_capacity(modification.modified_chunks_indices.len());

        for added_chunk_index in &modification.added_chunks_indices {
            let byte_index = added_chunk_index * chunk_size;
            let end = usize::min(byte_index + chunk_size, bytes.len());
            let added_bytes = &bytes[byte_index..end];
            
            additions.push(ModifiedChunk {
                index: *added_chunk_index,
                bytes: added_bytes.to_vec()
            });
        }

        for modified_chunk_index in &modification.modified_chunks_indices {
            let byte_index = modified_chunk_index * chunk_size;
            let end = usize::min(byte_index + chunk_size, bytes.len());
            let modified_bytes = &bytes[byte_index..end];
            
            modifications.push(ModifiedChunk {
                index: *modified_chunk_index,
                bytes: modified_bytes.to_vec()
            });
        }

        modified_files.push(ModifiedFile{
            file_path: modification.file_path.clone(),
            added_chunks: additions,
            deleted_chunks: modification.deleted_chunks_indices.clone(),
            modified_chunks: modifications
        });
    }

    Ok(PatchPackage {
        old_ver: plan.old_ver.clone(),
        new_ver: plan.new_ver.clone(),
        chunk_size: chunk_size,
        added_files: added_files,
        deleted_files: deleted_files,
        modified_files: modified_files
    })
}