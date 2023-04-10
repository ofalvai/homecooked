use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ignore::Walk;

use owo_colors::OwoColorize;
use tiktoken_rs::get_bpe_from_model;

use crate::{
    config::{self, Config},
    types::Embedding,
};

pub struct Note {
    pub title: String,
    pub path: PathBuf,
    pub text_content: String,
    // TODO: how to handle metadata, such as tags?
}

pub fn collect_notes(root: &PathBuf) -> Vec<Note> {
    collect_files(&root)
        .into_iter()
        .filter_map(|file| match file_to_note(&file, &root) {
            Ok(note) => Some(note),
            Err(err) => {
                println!("Failed to parse note: {}", file.clone().to_string_lossy());
                println!("Error: {}", err.red());
                None
            }
        })
        .collect()
}

pub fn token_count(text: &str) -> usize {
    let bpe = get_bpe_from_model(config::EMBEDDING_MODEL).unwrap();
    return bpe.encode_with_special_tokens(text).len();
}

pub fn note_to_input(note: &Note) -> String {
    let content = note.text_content.replace("\n", " ");
    let content = content.trim();
    if content.is_empty() {
        format!("{}", note.title)
    } else {
        format!("Note title: {}. Note content: {}", note.title, content)
    }
}

pub fn file_to_note(path: &Path, root_path: &Path) -> anyhow::Result<Note> {
    let title = path
        .file_stem()
        .expect("A file is supposed to have a name")
        .to_string_lossy();
    let text_content = std::fs::read_to_string(&path)?;
    let canonical_root = root_path.canonicalize()?;
    let relative_path = path
        .canonicalize()?
        .strip_prefix(canonical_root)?
        .to_path_buf();

    Ok(Note {
        title: title.to_string(),
        path: relative_path,
        text_content,
    })
}

pub fn note_to_checksum(note: &Note) -> u32 {
    crc32fast::hash(note.text_content.as_bytes())
}

fn collect_files(root: &PathBuf) -> Vec<PathBuf> {
    // TODO: configure ignore rules
    Walk::new(root)
        .filter_map(|result| {
            let entry = result.expect("Error iterating over files");
            let metadata = entry.metadata().unwrap();
            match metadata.is_dir() {
                true => None,
                false => Some(entry.path().to_path_buf()),
            }
        })
        .filter(|entry| {
            let extension = entry
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap();
            extension == "md"
        })
        .collect()
}

pub fn load_embeddings(config: &Config) -> anyhow::Result<Vec<Embedding>> {
    let embeddings_buf = fs::read(&config.embedding_path).context("Can't read embeddings file")?;
    let embeddings: Vec<Embedding> = rmp_serde::from_slice(&embeddings_buf)?;
    Ok(embeddings)
}
