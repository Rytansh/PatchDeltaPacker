use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, io};

use crate::build::concurrency::worker_pool::WorkerPool;
use crate::build::patcher::structs::{
    AddedFile, DeletedFile, Modification, ModifiedChunk, ModifiedFile, PatchPackage, PatchPlan,
};
use crate::build::{manifests, patcher};

pub async fn build_patch(
    old_patch_root: &Path,
    new_patch_root: &Path,
    worker_pool: &WorkerPool,
) -> Result<PatchPackage, io::Error> {
    let start = Instant::now();
    let old_manifest = match manifests::reader::get_manifest(old_patch_root) {
        Ok(manifest) => manifest,
        Err(_) => manifests::writer::generate_manifest(old_patch_root, worker_pool).await?,
    };

    let new_manifest = match manifests::reader::get_manifest(new_patch_root) {
        Ok(manifest) => manifest,
        Err(_) => manifests::writer::generate_manifest(new_patch_root, worker_pool).await?,
    };
    let elapsed = start.elapsed();
    println!("Manifest generation for patch took: {elapsed:?}.");
    let plan = patcher::plan_builder::create_patch_plan(old_manifest, new_manifest)?;
    let package = create_patch_package(&plan, new_patch_root, worker_pool).await?;

    Ok(package)
}

async fn create_patch_package(
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

        let handle = worker_pool.execute(move || build_added_file(file_path, &patch_root));
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
        let handle = worker_pool
            .execute(move || build_modified_file(&modification, &patch_root, chunk_size));
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

fn build_added_file(file_path: PathBuf, new_patch_root: &Path) -> Result<AddedFile, io::Error> {
    let bytes: Vec<u8> = fs::read(new_patch_root.join(&file_path))?;
    Ok(AddedFile {
        file_path,
        bytes_added: bytes,
    })
}

const fn build_deleted_file(file_path: PathBuf) -> DeletedFile {
    DeletedFile { file_path }
}

fn build_modified_file(
    modification: &Modification,
    new_patch_root: &Path,
    chunk_size: usize,
) -> Result<ModifiedFile, io::Error> {
    let mut file = File::open(new_patch_root.join(&modification.file_path))?;
    let target_file_size = file.metadata()?.len() as usize;

    let mut added_chunks = Vec::with_capacity(modification.added_chunks_indices.len());
    let mut modified_chunks = Vec::with_capacity(modification.modified_chunks_indices.len());

    let mut buffer = vec![0u8; chunk_size];

    let mut current_chunk = 0;

    let mut next_added = 0;
    let mut next_modified = 0;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let slice = &buffer[..bytes_read];

        if next_added < modification.added_chunks_indices.len()
            && modification.added_chunks_indices[next_added] == current_chunk
        {
            added_chunks.push(ModifiedChunk {
                index: current_chunk,
                bytes: slice.to_vec(),
            });

            next_added += 1;
        }

        if next_modified < modification.modified_chunks_indices.len()
            && modification.modified_chunks_indices[next_modified] == current_chunk
        {
            modified_chunks.push(ModifiedChunk {
                index: current_chunk,
                bytes: slice.to_vec(),
            });

            next_modified += 1;
        }

        current_chunk += 1;
    }

    Ok(ModifiedFile {
        file_path: modification.file_path.clone(),
        target_file_size,
        added_chunks,
        deleted_chunks: modification.deleted_chunks_indices.clone(),
        modified_chunks,
    })
}
