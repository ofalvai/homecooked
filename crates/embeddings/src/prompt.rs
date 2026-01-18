use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::Context;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};

use owo_colors::OwoColorize;
use urlencoding::encode;

use crate::{common::Note, config::Config, unlinked::UnlinkedPair};

pub struct PairListItem {
    pub path_a: PathBuf,
    pub path_b: PathBuf,
    pub similarity: f32,
}

impl fmt::Display for PairListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_a = self
            .path_a
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let title_b = self
            .path_b
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        let display_a = if title_a == title_b {
            self.path_a.display().to_string()
        } else {
            title_a.to_string()
        };
        let display_b = if title_a == title_b {
            self.path_b.display().to_string()
        } else {
            title_b.to_string()
        };

        write!(
            f,
            "{} ~ {} ({})",
            display_a.red(),
            display_b.green(),
            format!("{:.0}%", self.similarity * 100.0).yellow(),
        )
    }
}

pub struct NoteListItem {
    pub note_path: PathBuf,
    pub similarity: f32,
}

impl fmt::Display for NoteListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = self.note_path.file_stem().expect("File should have a name");
        let path = self.note_path.parent().unwrap_or(Path::new("."));
        write!(
            f,
            "{} {} {} [{}] ",
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
    mut selection_index: usize,
) -> anyhow::Result<()> {
    loop {
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
                selection_index = index;
            }
            None => {
                println!("Exiting");
                break;
            }
        }
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

pub fn unlinked_selector(pairs: Vec<UnlinkedPair>, config: &Config) -> anyhow::Result<()> {
    if pairs.is_empty() {
        println!("No unlinked pairs found");
        return Ok(());
    }

    let pair_items: Vec<PairListItem> = pairs
        .iter()
        .map(|p| PairListItem {
            path_a: p.path_a.clone(),
            path_b: p.path_b.clone(),
            similarity: p.similarity,
        })
        .collect();

    let mut index = 0usize;

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&pair_items)
            .default(index)
            .with_prompt("Select similar notes")
            .max_length(10)
            .interact_opt()?;

        match selection {
            Some(idx) => {
                index = idx;
            }
            None => {
                break;
            }
        }

        loop {
            let pair = &pairs[index];
            let pair_item = &pair_items[index];

            let options = vec!["Open A", "Open B", "Next", "Previous", "Back", "Quit"];
            let action_selection = Select::with_theme(&ColorfulTheme::default())
                .items(&options)
                .default(0)
                .with_prompt(pair_item.to_string())
                .report(false)
                .interact_opt()?;

            match action_selection {
                Some(0) => {
                    open_note(&config.vault, &pair.path_a)?;
                    continue;
                }
                Some(1) => {
                    open_note(&config.vault, &pair.path_b)?;
                    continue;
                }
                Some(2) => {
                    if index + 1 < pairs.len() {
                        index += 1;
                    }
                }
                Some(3) => {
                    index = index.saturating_sub(1);
                }
                Some(4) => {
                    break;
                }
                Some(5) => {
                    return Ok(());
                }
                None => {
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
