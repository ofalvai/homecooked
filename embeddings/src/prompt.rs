use std::path::{Path, PathBuf};
use std::fmt;

use anyhow::Context;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};

use owo_colors::OwoColorize;
use urlencoding::encode;

use crate::{common::Note, config::Config};

pub struct NoteListItem {
    pub note_path: PathBuf,
    pub similarity: f32,
}

impl fmt::Display for NoteListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = self.note_path.file_stem().expect("File should have a name");
        let path = self.note_path.parent().unwrap_or(Path::new("."));
        write!(f, "{} {} {} [{}] ",
            title.to_string_lossy().bold().bright_white(),
            "in".dimmed(),
            path.display().dimmed(),
            format!("{:.0}%", self.similarity * 100.0).green(),
        )
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.parent().unwrap_or(Path::new("."));
        write!(f, "{} - {}", self.title, path.display())
    }
}

pub fn result_selector(
    notes: Vec<NoteListItem>,
    config: &Config,
    selection_index: usize,
) -> anyhow::Result<()> {
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
        }
        None => println!("Exiting"),
    }

    Ok(())
}

pub fn prompt_query() -> anyhow::Result<String> {
    let input: String = Input::new().with_prompt("Search query").interact_text()?;
    Ok(input)
}

pub fn prompt_note_path(notes: &[Note]) -> anyhow::Result<&Note> {
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(notes)
        .default(0)
        .with_prompt("Select note from vault")
        .max_length(10)
        .interact_opt()?;

    match selection {
        Some(index) => Ok(&notes[index]),
        None => anyhow::bail!("No note selected"),
    }
}

fn open_note(vault_name: &str, note_path: &Path) -> anyhow::Result<()> {
    open::that(obsidian_uri(vault_name, note_path)).context("Failed to launch Obsidian URI")?;
    Ok(())
}

fn obsidian_uri(vault_name: &str, note_path: &Path) -> String {
    let path_str = note_path.to_string_lossy();
    let encoded_path = encode(&path_str);
    format!("obsidian://open?vault={vault_name}&file={encoded_path}")
}
