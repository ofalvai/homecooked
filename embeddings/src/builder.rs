use anyhow::Context;
use async_openai::config::OpenAIConfig;
use async_openai::{types::CreateEmbeddingRequestArgs, Client};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::common::{collect_notes, note_to_checksum, note_to_inputs, Note};
use crate::config::{self, Config};
use crate::types::Embedding;

pub async fn build(config: &Config, dry_run: bool) -> anyhow::Result<()> {
    let notes = collect_notes(&config.notes_root);

    let mut embeddings =
        load_embeddings(&config.embedding_path).context("Failed to load embeddings")?;

    let client = Client::with_config(OpenAIConfig::new().with_api_key(config.api_key.clone()));

    for (i, note) in notes.into_iter().enumerate() {
        let checksum = note_to_checksum(&note);
        let stored_embedding = embeddings.get(&note.path);
        if let Some(stored_embedding) = stored_embedding {
            if stored_embedding.note_checksum == checksum {
                continue;
            }
            println!("{} {}", "Updating".yellow(), note.path.to_string_lossy());
        } else {
            println!("{} {}", "Create".blue(), note.path.to_string_lossy());
        }

        if dry_run {
            println!("Note: {}", note.path.to_string_lossy().yellow());
            println!("Checksum: {}", note_to_checksum(&note).yellow());
            println!();
            continue;
        }

        match build_embeddings(&client, &note).await {
            Ok(result) => {
                for embedding in result {
                    embeddings.insert(
                        note.path.to_owned(),
                        Embedding {
                            note_path: note.path.to_owned(),
                            embedding,
                            note_checksum: checksum,
                        },
                    );
                }
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

        if (i + 1) % 10 == 0 {
            println!(
                "{} Persisting {} note embeddings",
                "Checkpoint".purple(),
                embeddings.len()
            );
            save_embeddings(&embeddings, &config.embedding_path)
                .context("Failed to save embeddings")?;
        }
    }

    save_embeddings(&embeddings, &config.embedding_path).context("Failed to save embeddings")?;

    Ok(())
}

async fn build_embeddings(
    client: &Client<OpenAIConfig>,
    note: &Note,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let mut embeddings = vec![];
    for input in note_to_inputs(note) {
        let embedding = get_embedding(client, input).await?;
        embeddings.push(embedding);
    }
    Ok(embeddings)
}

async fn get_embedding(client: &Client<OpenAIConfig>, input: String) -> anyhow::Result<Vec<f32>> {
    let request = CreateEmbeddingRequestArgs::default()
        .model(config::EMBEDDING_MODEL)
        .input(input)
        .build()?;

    let response = client.embeddings().create(request).await?;
    let embedding = response
        .data
        .get(0)
        .context("No embedding returned")?
        .embedding
        .to_owned();
    Ok(embedding)
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

pub fn prune(config: &Config) -> anyhow::Result<()> {
    let notes: HashMap<PathBuf, Note> = collect_notes(&config.notes_root)
        .into_iter()
        .map(|note| (note.path.clone(), note))
        .collect();

    let mut embeddings =
        load_embeddings(&config.embedding_path).context("Failed to load embeddings")?;

    let mut removed_count = 0;

    embeddings.retain(|embedding_path, embedding| {
        if !notes.contains_key(embedding_path) {
            println!(
                "{} {}",
                "Remove".red(),
                embedding.note_path.to_string_lossy()
            );
            removed_count += 1;
            false
        } else {
            true
        }
    });

    save_embeddings(&embeddings, &config.embedding_path).context("Failed to save embeddings")?;

    println!();
    if removed_count > 0 {
        println!("Sucessfully pruned {} embeddings", removed_count);
    } else {
        println!("There is nothing to prune at the moment.")
    }

    Ok(())
}
