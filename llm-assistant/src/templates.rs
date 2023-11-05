use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Template {
    pub id: String,
    pub prompt: String,
    pub label: String,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Templates {
    pub templates: Vec<Template>,
}

pub fn read_template(template_file: &Path, id: String) -> anyhow::Result<Template> {
    let templates = std::fs::read_to_string(template_file)?;
    let templates: Templates = serde_yaml::from_str(&templates)?;

    match templates.templates.into_iter().find(|t| t.id == id) {
        Some(template) => Ok(template),
        None => anyhow::bail!("Template with ID {} not found", id),
    }
}
