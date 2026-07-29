use crate::build::concurrency::worker_pool::WorkerPool;
use crate::build::config::config_reader;
use crate::build::patcher::patch_structs::{AddedFile, DeletedFile, ModifiedFile, PatchPackage};
use std::fs;
use std::io;
use std::path::Path;

pub async fn install_patch(
    patch: PatchPackage,
    worker_pool: &WorkerPool,
    game_directory: &Path,
) -> Result<(), io::Error> {
    if config_reader::get_game_version(game_directory)? != patch.old_ver {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Version mismatch. Cannot download patch.",
        ));
    }

    let chunk_size = patch.chunk_size;

    let mut mod_handles = Vec::new();
    let mut add_handles = Vec::new();
    let mut del_handles = Vec::new();
    for modified_file in patch.modified_files {
        let directory = game_directory.to_path_buf();
        let mod_handle =
            worker_pool.execute(move || modify_file(&modified_file, &directory, chunk_size));
        mod_handles.push(mod_handle);
    }
    for handle in mod_handles {
        handle.wait().await?;
    }
    for added_file in patch.added_files {
        let directory = game_directory.to_path_buf();
        let add_handle = worker_pool.execute(move || add_file(&added_file, &directory));
        add_handles.push(add_handle);
    }
    for handle in add_handles {
        handle.wait().await?;
    }
    for deleted_file in patch.deleted_files {
        let directory = game_directory.to_path_buf();
        let del_handle = worker_pool.execute(move || delete_file(&deleted_file, &directory));
        del_handles.push(del_handle);
    }
    for handle in del_handles {
        handle.wait().await?;
    }

    Ok(())
}

pub fn delete_file(file: &DeletedFile, game_directory: &Path) -> Result<(), io::Error> {
    let path = game_directory.join(&file.file_path);
    Ok(fs::remove_file(path)?)
}

pub fn add_file(file: &AddedFile, game_directory: &Path) -> Result<(), io::Error> {
    let path = game_directory.join(&file.file_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, &file.bytes_added)?)
}

pub fn modify_file(
    file: &ModifiedFile,
    game_directory: &Path,
    chunk_size: usize,
) -> Result<(), io::Error> {
    let path = game_directory.join(&file.file_path);
    let mut file_bytes = fs::read(&path)?;

    if file_bytes.len() < file.target_file_size {
        file_bytes.resize(file.target_file_size, 0);
    }

    for modified_chunk in &file.modified_chunks {
        let start = modified_chunk.index * chunk_size;
        let end = start + modified_chunk.bytes.len();

        file_bytes[start..end].copy_from_slice(&modified_chunk.bytes);
    }

    for added_chunk in &file.added_chunks {
        let start = added_chunk.index * chunk_size;
        let end = start + added_chunk.bytes.len();

        file_bytes[start..end].copy_from_slice(&added_chunk.bytes);
    }

    file_bytes.truncate(file.target_file_size);

    fs::write(path, file_bytes)?;

    Ok(())
}
