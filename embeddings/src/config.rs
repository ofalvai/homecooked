use std::path::PathBuf;

use anyhow::Context;
use configparser::ini::Ini;
use directories::ProjectDirs;

pub const EMBEDDING_MODEL: &str = "text-embedding-ada-002";
pub const MAX_TOKENS: usize = 8191;
pub const COST_PER_TOKEN: f64 = 0.0004 / 1000.0;

const EMBEDDING_FILE: &str = "embeddings.msgpack";

pub struct Config {
    pub api_key: String,
    pub notes_root: PathBuf,
    pub embedding_path: PathBuf,
}

pub fn load_config() -> anyhow::Result<Config> {
    let project_dirs = ProjectDirs::from("com.ofalvai.homecooked", "", "embeddings")
        .context("Can't find config directory")?;
    let config_path = project_dirs.config_dir().join("config.ini");
    let mut config = Ini::new();
    let _ = config.load(config_path);
    let api_key = config
        .get("openai", "api_key")
        .context("Can't find api_key in config.ini")?;
    let notes_root = config
        .get("notes", "root")
        .context("Can't find notes root in config.ini")?;
    let notes_path = PathBuf::from(shellexpand::tilde(&notes_root).to_string())
        .canonicalize()
        .context("Invalid note root path")?;

    Ok(Config {
        api_key,
        notes_root: notes_path,
        embedding_path: project_dirs.data_dir().join(EMBEDDING_FILE),
    })
}
