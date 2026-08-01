use std::fs::{self, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::build::{
    concurrency::worker_pool::WorkerPool,
    config,
    patcher::structs::{AddedFile, DeletedFile, ModifiedFile, PatchPackage},
};
use crate::client::installation::{progress::UpdateProgress, structs::PreparedFile};
use crate::constants::{BACKUP_EXTENSION, TEMPORARY_PATCH_EXTENSION, TEMPORARY_PATCH_PATH};

pub async fn install_patch(
    patch: PatchPackage,
    worker_pool: &WorkerPool,
    game_directory: &Path,
    progress: Arc<UpdateProgress>,
) -> Result<(), io::Error> {
    recover_interrupted_install(game_directory)?;

    if config::reader::get_game_version(game_directory)? != patch.old_ver {
        return Err(io::Error::other("Version mismatch. Cannot install patch."));
    }

    let total_operations =
        patch.modified_files.len() + patch.added_files.len() + patch.deleted_files.len();

    progress.begin_install(total_operations);

    let chunk_size = patch.chunk_size;

    let mut add_handles = Vec::new();
    let mut modify_handles = Vec::new();
    let mut prepared_files = Vec::new();

    for modified_file in patch.modified_files {
        let directory = game_directory.to_path_buf();

        let progress = Arc::clone(&progress);

        let handle = worker_pool.execute(move || {
            let prepared = prepare_modified_file(&modified_file, &directory, chunk_size)?;

            progress.complete_install_operation();

            Ok::<PreparedFile, io::Error>(prepared)
        });

        modify_handles.push(handle);
    }

    for added_file in patch.added_files {
        let directory = game_directory.to_path_buf();
        let progress = Arc::clone(&progress);

        let handle = worker_pool.execute(move || {
            let prepared = prepare_added_file(&added_file, &directory)?;

            progress.complete_install_operation();

            Ok::<PreparedFile, io::Error>(prepared)
        });

        add_handles.push(handle);
    }

    for handle in modify_handles {
        prepared_files.push(handle.wait().await?);
    }

    for handle in add_handles {
        prepared_files.push(handle.wait().await?);
    }

    prepared_files.sort_by(|a, b| a.original.cmp(&b.original));

    commit_prepared_files(&prepared_files)?;

    let mut delete_handles = Vec::new();

    for deleted_file in patch.deleted_files {
        let directory = game_directory.to_path_buf();
        let progress = Arc::clone(&progress);

        let handle = worker_pool.execute(move || {
            delete_file(&deleted_file, &directory)?;

            progress.complete_install_operation();

            Ok::<(), io::Error>(())
        });

        delete_handles.push(handle);
    }

    for handle in delete_handles {
        handle.wait().await?;
    }

    cleanup_backups(&prepared_files)?;

    Ok(())
}

pub fn delete_file(file: &DeletedFile, game_directory: &Path) -> Result<(), io::Error> {
    let path = game_directory.join(&file.file_path);
    if path.exists() {
        fs::remove_file(path)?;
        return Ok(());
    }
    Ok(())
}

fn prepare_modified_file(
    file: &ModifiedFile,
    game_directory: &Path,
    chunk_size: usize,
) -> Result<PreparedFile, io::Error> {
    let original = game_directory.join(&file.file_path);
    let temp = temp_path(&original);

    fs::copy(&original, &temp)?;

    let mut output = OpenOptions::new().read(true).write(true).open(&temp)?;

    if output.metadata()?.len() < file.target_file_size as u64 {
        output.set_len(file.target_file_size as u64)?;
    }

    for modified_chunk in &file.modified_chunks {
        let offset = (modified_chunk.index * chunk_size) as u64;

        output.seek(SeekFrom::Start(offset))?;
        output.write_all(&modified_chunk.bytes)?;
    }

    for added_chunk in &file.added_chunks {
        let offset = (added_chunk.index * chunk_size) as u64;

        output.seek(SeekFrom::Start(offset))?;
        output.write_all(&added_chunk.bytes)?;
    }

    output.set_len(file.target_file_size as u64)?;

    output.flush()?;

    Ok(PreparedFile { original, temp })
}

fn prepare_added_file(file: &AddedFile, game_directory: &Path) -> Result<PreparedFile, io::Error> {
    let original = game_directory.join(&file.file_path);
    let temp = temp_path(&original);

    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&temp, &file.bytes_added)?;

    Ok(PreparedFile { original, temp })
}

fn commit_prepared_files(prepared_files: &[PreparedFile]) -> Result<(), io::Error> {
    let mut committed: Vec<&PreparedFile> = Vec::new();

    for file in prepared_files {
        let backup = backup_path(&file.original);

        if file.original.exists()
            && let Err(err) = fs::rename(&file.original, &backup)
        {
            rollback_committed_files(&committed)?;
            return Err(err);
        }

        if let Err(err) = fs::rename(&file.temp, &file.original) {
            recover_failed_swap(file)?;

            rollback_committed_files(&committed)?;

            return Err(err);
        }

        committed.push(file);
    }

    Ok(())
}

fn cleanup_backups(prepared_files: &[PreparedFile]) -> Result<(), io::Error> {
    for file in prepared_files {
        let backup = backup_path(&file.original);

        if backup.exists() {
            fs::remove_file(backup)?;
        }

        if file.temp.exists() {
            fs::remove_file(&file.temp)?;
        }
    }

    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().unwrap().to_string_lossy();

    path.with_file_name(format!("{file_name}.{TEMPORARY_PATCH_EXTENSION}"))
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().unwrap().to_string_lossy();

    path.with_file_name(format!("{file_name}.{BACKUP_EXTENSION}"))
}
fn rollback_committed_files(committed: &[&PreparedFile]) -> Result<(), io::Error> {
    for file in committed.iter().rev() {
        let backup = backup_path(&file.original);

        if file.original.exists() {
            let _ = fs::remove_file(&file.original);
        }

        if backup.exists() {
            fs::rename(&backup, &file.original)?;
        }

        if file.temp.exists() {
            let _ = fs::remove_file(&file.temp);
        }
    }

    Ok(())
}

fn recover_failed_swap(file: &PreparedFile) -> Result<(), io::Error> {
    let backup = backup_path(&file.original);

    if !file.original.exists() && backup.exists() {
        fs::rename(backup, &file.original)?;
    }

    Ok(())
}

fn recover_interrupted_install(game_directory: &Path) -> Result<(), io::Error> {
    fn recurse(dir: &Path) -> Result<(), io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;

            let path = entry.path();

            if path.is_dir() {
                recurse(&path)?;
                continue;
            }

            let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };

            if extension.ends_with(BACKUP_EXTENSION) {
                let original = remove_suffix(&path, BACKUP_EXTENSION);

                if original.exists() {
                    fs::remove_file(&original)?;
                }

                fs::rename(&path, original)?;

                continue;
            }

            if extension.ends_with(TEMPORARY_PATCH_PATH) {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    recurse(game_directory)
}

fn remove_suffix(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path.file_name().unwrap().to_string_lossy();

    let stripped = file_name
        .strip_suffix(&format!(".{suffix}"))
        .expect("Expected suffix");

    path.with_file_name(stripped)
}
