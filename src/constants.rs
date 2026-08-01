pub const COMPRESSION_LEVEL: i32 = 3;
pub const MANIFEST_VERSION: &str = "1.0.0";
pub const CHUNK_SIZE: usize = 1024 * 1024;
pub const HASH_SEED: u64 = 1;

pub const MANIFEST_RELATIVE_PATH: &str = "manifest.json";
pub const CONFIG_RELATIVE_PATH: &str = "GameConfig.json";
pub const PATCH_HISTORY_RELATIVE_PATH: &str = "patch_history.json";

pub const TEMPORARY_PATCH_PATH: &str = r"temp\patch.tmp";
pub const TEMPORARY_PATCH_EXTENSION: &str = "patch";
pub const BACKUP_EXTENSION: &str = "bak";
