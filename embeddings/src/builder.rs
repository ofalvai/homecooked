use anyhow::Context;
use async_openai::{types::CreateEmbeddingRequestArgs, Client};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::common::{collect_notes, note_to_input, token_count, Note, note_to_checksum};
use crate::config::{self, Config};
use crate::types::Embedding;

pub async fn build(config: &Config, dry_run: bool) -> anyhow::Result<()> {
    let notes = collect_notes(&config.notes_root);

    let mut embeddings =
        load_embeddings(&config.embedding_path).context("Failed to load embeddings")?;

    let client = Client::new().with_api_key(config.api_key.to_owned());

    for (i, note) in notes.into_iter().enumerate() {
        let checksum = note_to_checksum(&note);
        let stored_embedding = embeddings.get(&note.path);
        if let Some(stored_embedding) = stored_embedding {
            if stored_embedding.note_checksum == checksum {
                // println!("{} {}", "No change".green(), note.path.to_string_lossy());
                continue;
            }
            println!("{} {}", "Updating".yellow(), note.path.to_string_lossy());
        } else {
            println!("{} {}", "Create".blue(), note.path.to_string_lossy());
        }

        if dry_run {
            println!("Note: {}", note.path.to_string_lossy().yellow());
            println!("Checksum: {}", note_to_checksum(&note).yellow());
            // println!("{}", note_to_input(&note));
            println!();
            continue;
        }

        match get_embedding(&client, &note).await {
            Ok(result) => {
                embeddings.insert(
                    note.path.clone(),
                    Embedding {
                        note_path: note.path.to_owned(),
                        embedding: result,
                        note_checksum: checksum,
                    },
                );
            }
            Err(err) => {
                println!(
                    "{} {}",
                    "Failed to get embedding for".red(),
                    note.path.to_string_lossy()
                );
                println!("Error: {}", err.red());
            }
        }

        if i % 11 == 0 {
            println!(
                "{} {}",
                "Checkpoint".purple(),
                format!("Persisting {} note embeddings", embeddings.len())
            );
            save_embeddings(&embeddings, &config.embedding_path)
                .context("Failed to save embeddings")?;
        }
    }

    save_embeddings(&embeddings, &config.embedding_path).context("Failed to save embeddings")?;

    Ok(())
}

async fn get_embedding(client: &Client, note: &Note) -> anyhow::Result<Vec<f32>> {
    if token_count(&note.text_content) > config::MAX_TOKENS {
        return Err(anyhow::anyhow!("Note is too long"));
    }

    let request = CreateEmbeddingRequestArgs::default()
        .model(config::EMBEDDING_MODEL)
        .input(note_to_input(note))
        .build()?;

    let response = client.embeddings().create(request).await?;
    Ok(response.data.get(0).unwrap().embedding.to_owned())
}

fn load_embeddings(path: &PathBuf) -> anyhow::Result<HashMap<PathBuf, Embedding>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let buf = std::fs::read(path)?;
    let embeddings: Vec<Embedding> = rmp_serde::from_slice(&buf)?;

    let embedding_map = embeddings
        .into_iter()
        .map(|embedding| (embedding.note_path.clone(), embedding))
        .collect();

    Ok(embedding_map)
}

fn save_embeddings(embeddings: &HashMap<PathBuf, Embedding>, path: &PathBuf) -> anyhow::Result<()> {
    let embedding_list: Vec<Embedding> = embeddings
        .iter()
        .map(|(_, embedding)| embedding.clone())
        .collect();

    let buf = rmp_serde::to_vec(&embedding_list)?;
    std::fs::write(path, buf)?;

    Ok(())
}
