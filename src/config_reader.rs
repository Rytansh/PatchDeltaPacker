use std::fs;
use std::io;
use serde_json;
use serde;
use std::path::{Path, PathBuf};

const GAME_CONFIG_PATH: &str = "GameConfig.json";

#[derive(serde::Deserialize)]
pub struct GameConfig {
    pub gameVersion: String
}

pub fn get_game_version(root_directory_path: &Path) -> Result<String, io::Error>
{
    let config = get_config(root_directory_path)?;
    Ok(config.gameVersion)
}

fn get_config(root_directory_path: &Path) -> Result<GameConfig, io::Error>
{
    let game_config_path : PathBuf = root_directory_path.join(Path::new(GAME_CONFIG_PATH));
    let config_text = fs::read_to_string(game_config_path)?;
    let config : GameConfig = serde_json::from_str(&config_text)?;

    Ok(config)
}