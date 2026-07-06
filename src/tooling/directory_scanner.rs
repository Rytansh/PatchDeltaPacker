use std::fs;
use std::io;
use std::path::{Path,PathBuf};

pub fn scan_directory(root_directory_path: &Path) -> Result<Vec<PathBuf>, io::Error>
{
    let mut all_paths = Vec::new();
    for entry_result in fs::read_dir(root_directory_path)? //returns Vec<DirEntry> if no errors encountered
    {
        let file_entry = entry_result?;
        let type_file = file_entry.metadata()?.file_type();
        if type_file.is_file()
        {
            all_paths.push(file_entry.path())
        } else if type_file.is_dir()
        {
            for recursive_entry in scan_directory(&file_entry.path())? {
                all_paths.push(recursive_entry);
            }
        }
    }

    Ok(all_paths)
}
