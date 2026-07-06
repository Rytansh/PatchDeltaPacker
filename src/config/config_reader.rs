use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::constants::CONFIG_RELATIVE_PATH;

use serde_json;
use serde;

#[derive(serde::Deserialize)]
pub struct GameConfig {
    pub game_version: String
}

pub fn get_game_version(root_directory_path: &Path) -> Result<String, io::Error>
{
    let config = get_config(root_directory_path)?;
    Ok(config.game_version)
}

fn get_config(root_directory_path: &Path) -> Result<GameConfig, io::Error>
{
    let game_config_path : PathBuf = root_directory_path.join(Path::new(CONFIG_RELATIVE_PATH));
    let config_text = fs::read_to_string(game_config_path)?;
    let config : GameConfig = serde_json::from_str(&config_text)?;

    Ok(config)
}