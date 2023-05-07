use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use configparser::ini::Ini;
use directories::ProjectDirs;

pub const EMBEDDING_MODEL: &str = "text-embedding-ada-002";
pub const MAX_TOKENS: usize = 8191;
pub const EMBEDDING_DIM: usize = 1536;
pub const COST_PER_TOKEN: f64 = 0.0004 / 1000.0;

const EMBEDDING_FILE: &str = "embeddings.msgpack";

pub struct Config {
    pub api_key: String,
    pub notes_root: PathBuf,
    pub vault: String,
    pub embedding_path: PathBuf,
    pub plot_colors: HashMap<String, String>,
}

pub fn load_config() -> anyhow::Result<Config> {
    let project_dirs = ProjectDirs::from("com.oliverfalvai.homecooked", "", "embeddings")
        .context("Can't find config directory")?;
    let config_path = project_dirs.config_dir().join("config.ini");
    let mut config = Ini::new_cs(); // case sensitive because of plot colors and paths
    let config_map = config.load(config_path).map_err(|err| {
        anyhow::anyhow!("Failed to load config file: {}", err)
    })?;
    let api_key = config
        .get("openai", "api_key")
        .context("Can't find api_key field in config.ini")?;
    let vault = config
        .get("notes", "vault")
        .context("Can't find vault field in config.ini")?;
    let notes_root = config
        .get("notes", "root")
        .context("Can't find root field in config.ini")?;
    let notes_path = PathBuf::from(shellexpand::tilde(&notes_root).to_string())
        .canonicalize()
        .context("Invalid note root path")?;

    let plot_colors = config_map
        .get("plot_colors")
        .unwrap_or(&HashMap::new())
        .iter()
        .filter_map(|(k, v)| match (k, v) {
            (k, Some(v)) => Some((k.to_string(), v.to_string())),
            _ => None,
        })
        .collect();

    Ok(Config {
        api_key,
        notes_root: notes_path,
        vault,
        embedding_path: project_dirs.data_dir().join(EMBEDDING_FILE),
        plot_colors,
    })
}
