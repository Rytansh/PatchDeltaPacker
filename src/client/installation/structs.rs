use std::path::PathBuf;

#[derive(Debug)]
pub struct PreparedFile {
    pub original: PathBuf,
    pub temp: PathBuf,
}
