use crate::concurrency::worker_pool::WorkerPool;
use crate::manifests::manifest_reader;
use crate::manifests::manifest_ser::write_manifest;
use crate::patcher::patch_plan_gen::create_patch_plan;
use crate::patcher::patch_structs::{
    AddedFile, DeletedFile, Modification, ModifiedChunk, ModifiedFile, PatchPackage, PatchPlan,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub async fn build_patch(
    old_patch_root: &Path,
    new_patch_root: &Path,
    worker_pool: &WorkerPool,
) -> Result<PatchPackage, io::Error> {
    let old_manifest = match manifest_reader::get_manifest(old_patch_root) {
        Ok(manifest) => manifest,
        Err(error) => write_manifest(old_patch_root, worker_pool).await?,
    };

    let new_manifest = match manifest_reader::get_manifest(new_patch_root) {
        Ok(manifest) => manifest,
        Err(error) => write_manifest(new_patch_root, worker_pool).await?,
    };
    let plan = create_patch_plan(old_manifest, new_manifest)?;
    let package = create_patch_package(&plan, new_patch_root, worker_pool).await?;

    Ok(package)
}

pub async fn create_patch_package(
    plan: &PatchPlan,
    new_patch_root: &Path,
    worker_pool: &WorkerPool,
) -> Result<PatchPackage, io::Error> {
    let mut add_handles = Vec::new();
    let mut modify_handles = Vec::new();

    //ADDED FILES
    for file in &plan.added_files {
        let file_path = file.file_path.clone();
        let patch_root = new_patch_root.to_path_buf();

        let handle = worker_pool.execute(move || build_added_file(file_path, patch_root));
        add_handles.push(handle);
    }

    //DELETED FILES
    let deleted_files = plan
        .deleted_files
        .iter()
        .map(|file| build_deleted_file(file.file_path.clone()))
        .collect();

    //MODIFIED FILES
    for modification in &plan.modified_files {
        let patch_root = new_patch_root.to_path_buf();
        let chunk_size = plan.chunk_size;
        let modification = modification.clone();
        let handle =
            worker_pool.execute(move || build_modified_file(modification, patch_root, chunk_size));
        modify_handles.push(handle);
    }

    let mut added_files: Vec<AddedFile> = Vec::with_capacity(plan.added_files.len());
    for handle in add_handles {
        let added_file = handle.wait().await?;
        added_files.push(added_file);
    }

    let mut modified_files: Vec<ModifiedFile> = Vec::with_capacity(plan.modified_files.len());
    for handle in modify_handles {
        let modified_file = handle.wait().await?;
        modified_files.push(modified_file);
    }

    Ok(PatchPackage {
        old_ver: plan.old_ver.clone(),
        new_ver: plan.new_ver.clone(),
        chunk_size: plan.chunk_size,
        added_files,
        deleted_files,
        modified_files,
    })
}

fn build_added_file(file_path: PathBuf, new_patch_root: PathBuf) -> Result<AddedFile, io::Error> {
    let bytes: Vec<u8> = fs::read(new_patch_root.join(&file_path))?;
    println!(
        "[{:?}] ADDING {}",
        std::thread::current().id(),
        file_path.display()
    );
    Ok(AddedFile {
        file_path,
        bytes_added: bytes,
    })
}

const fn build_deleted_file(file_path: PathBuf) -> DeletedFile {
    DeletedFile { file_path }
}
fn build_modified_file(
    modification: Modification,
    new_patch_root: PathBuf,
    chunk_size: usize,
) -> Result<ModifiedFile, io::Error> {
    let bytes: Vec<u8> = fs::read(new_patch_root.join(&modification.file_path))?;

    println!(
        "[{:?}] MODIFYING {}",
        std::thread::current().id(),
        modification.file_path.display()
    );

    let mut additions: Vec<ModifiedChunk> =
        Vec::with_capacity(modification.added_chunks_indices.len());
    let mut modifications: Vec<ModifiedChunk> =
        Vec::with_capacity(modification.modified_chunks_indices.len());

    for added_chunk_index in &modification.added_chunks_indices {
        let byte_index = added_chunk_index * chunk_size;
        let end = usize::min(byte_index + chunk_size, bytes.len());
        let added_bytes = &bytes[byte_index..end];

        additions.push(ModifiedChunk {
            index: *added_chunk_index,
            bytes: added_bytes.to_vec(),
        });
    }

    for modified_chunk_index in &modification.modified_chunks_indices {
        let byte_index = modified_chunk_index * chunk_size;
        let end = usize::min(byte_index + chunk_size, bytes.len());
        let modified_bytes = &bytes[byte_index..end];

        modifications.push(ModifiedChunk {
            index: *modified_chunk_index,
            bytes: modified_bytes.to_vec(),
        });
    }

    Ok(ModifiedFile {
        file_path: modification.file_path.clone(),
        added_chunks: additions,
        deleted_chunks: modification.deleted_chunks_indices.clone(),
        modified_chunks: modifications,
    })
}
