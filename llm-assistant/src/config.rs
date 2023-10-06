use std::path::PathBuf;

use anyhow::Context;
use configparser::ini::Ini;
use directories::ProjectDirs;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub readwise_api_key: String,

    pub persona_file: PathBuf,
    pub template_dir: PathBuf,
}

pub fn load_config(path: Option<String>) -> anyhow::Result<Config> {
    let config_path = match path {
        Some(path) => PathBuf::from(path),
        None => {
            let project_dirs =
                ProjectDirs::from("com.oliverfalvai.homecooked", "", "llm-assistant")
                    .context("Can't find config directory")?;
            project_dirs.config_dir().join("config.ini")
        }
    };
    let mut config = Ini::new();
    config
        .load(config_path.clone())
        .map_err(|err| anyhow::anyhow!("Failed to load config file: {}", err))?;
    let openai_api_key = config
        .get("keys", "openai_api_key")
        .context("Can't find openai_api_key field in config.ini")?;
    let anthropic_api_key = config
        .get("keys", "anthropic_api_key")
        .context("Can't find anthropic_api_key field in config.ini")?;
    let readwise_api_key = config
        .get("keys", "readwise_api_key")
        .context("Can't find readwise_api_key field in config.ini")?;
    let persona_file = config
        .get("content", "persona_file")
        .context("Can't find persona_file field in config.ini")?;
    let template_dir = config
        .get("content", "template_dir")
        .context("Can't find template_dir field in config.ini")?;

    Ok(Config {
        openai_api_key,
        anthropic_api_key,
        readwise_api_key,
        persona_file: load_config_path(&config_path, &persona_file),
        template_dir: load_config_path(&config_path, &template_dir),
    })
}

fn load_config_path(config_path: &PathBuf, path_str: &str) -> PathBuf {
    let path_buf = PathBuf::from(path_str);

    if path_buf.is_absolute() {
        path_buf
    } else {
        config_path.parent().unwrap().join(path_buf)
    }
}
