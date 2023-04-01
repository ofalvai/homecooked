use std::path::{Path, PathBuf};

use anyhow::Context;
use dialoguer::{theme::ColorfulTheme, Input, Select, FuzzySelect};

use owo_colors::OwoColorize;
use urlencoding::encode;

use crate::{config::Config, common::Note};

pub struct NoteListItem {
    pub note_path: PathBuf,
    pub similarity: f32,
}

impl ToString for NoteListItem {
    fn to_string(&self) -> String {
        let title = self.note_path.file_stem().expect("File should have a name");
        let path = self.note_path.parent().unwrap_or(Path::new("."));
        format!(
            "{} {} {} [{}] ",
            title.to_string_lossy().bold().bright_white(),
            "in".dimmed(),
            path.display().dimmed(),
            format!("{:.0}%", self.similarity * 100.0).green(),
        )
    }
}

impl ToString for Note {
    fn to_string(&self) -> String {
        let path = self.path.parent().unwrap_or(Path::new("."));
        format!(
            "{} - {}",
            self.title,
            path.display(),
        )
    }
}

pub fn result_selector(notes: Vec<NoteListItem>, config: &Config, selection_index: usize) -> anyhow::Result<()> {
    let prompt = format!("Select note to open or {} to quit", "ESC".green());
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&notes)
        .default(selection_index)
        .with_prompt(prompt)
        .report(false)
        .interact_opt()?;

    match selection {
        Some(index) => {
            let path = notes[index].note_path.as_path();
            open_note(&config.vault, path)?;

            result_selector(notes, config, index)?;
        },
        None => println!("Exiting"),
    }

    Ok(())
}

pub fn prompt_query() -> anyhow::Result<String> {
    let input: String = Input::new()
        .with_prompt("Search query")
        .interact_text()?;

    return Ok(input);
}

pub fn prompt_note_path(notes: &Vec<Note>) -> anyhow::Result<&Note> {
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(notes)
        .default(0)
        .with_prompt("Select note from vault")
        .max_length(10)
        .interact_opt()?
        .unwrap();

    Ok(&notes[selection])
}

fn open_note(vault_name: &str, note_path: &Path) -> anyhow::Result<()> {
    open::that(obsidian_uri(vault_name, note_path)).context("Failed to launch Obsidian URI")?;
    Ok(())
}

fn obsidian_uri(vault_name: &str, note_path: &Path) -> String {
    let path_str = note_path.to_string_lossy();
    let encoded_path = encode(&path_str);
    format!("obsidian://open?vault={}&file={}", vault_name, encoded_path)
}