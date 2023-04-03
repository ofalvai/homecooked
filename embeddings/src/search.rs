use std::{fs, path::PathBuf, str::FromStr, time::Instant};

use anyhow::Context;
use owo_colors::OwoColorize;

use async_openai::{types::CreateEmbeddingRequestArgs, Client};
use try_partialord::TrySort;

use crate::{
    common::{collect_notes, file_to_note, note_to_checksum},
    config::{self, Config},
    prompt::{prompt_note_path, prompt_query, result_selector, NoteListItem},
    types::Embedding,
};

pub async fn query(config: &Config, query: Option<&str>) -> anyhow::Result<()> {
    let query = match query {
        Some(q) => q.to_owned(),
        None => prompt_query()?,
    };

    println!("Embedding query...");
    let embedding_start = Instant::now();
    let query_embedding = get_query_embedding(&config.api_key, &query).await?;
    let embedding_duration = embedding_start.elapsed();
    println!("Done");

    let parse_start = Instant::now();

    let mut embeddings: Vec<Embedding> =
        load_embeddings(&config).context("Failed to load embeddings from file")?;
    let parse_duration = parse_start.elapsed();

    let sort_start = Instant::now();
    embeddings
        .try_sort_by_cached_key(|e| Some(-cosine_similarity(&e.embedding, &query_embedding)))?;
    let sort_duration = sort_start.elapsed();

    println!();
    println!("Query embedding time: {:?}", embedding_duration.green());
    println!("Parse time: {:?}", parse_duration.green());
    println!("Sort time: {:?}", sort_duration.green());
    println!("Note count: {}", embeddings.len().to_string().green());

    println!();
    println!("Best matches for {}:", query.yellow());

    let items = embeddings
        .iter()
        .take(10)
        .map(|e| NoteListItem {
            note_path: e.note_path.clone(),
            similarity: cosine_similarity(&e.embedding, &&query_embedding),
        })
        .collect();
    result_selector(items, config, 0)?;

    Ok(())
}

pub fn related(config: &Config, note_path: &Option<String>) -> anyhow::Result<()> {
    let note_path = match note_path {
        Some(path) => PathBuf::from_str(path)?,
        None => {
            let notes = collect_notes(&config.notes_root);
            let selected = prompt_note_path(&notes).context("Error while selecting note")?;
            selected.path.to_owned()
        }
    };

    let display_path = note_path.to_str().unwrap();
    let note_path = if note_path.ends_with(".md") {
        note_path.to_path_buf()
    } else {
        note_path.with_extension("md")
    };
    let abs_path = config.notes_root.join(note_path.clone());
    let note = file_to_note(&abs_path, &config.notes_root)?;

    let mut embeddings: Vec<Embedding> =
        load_embeddings(&config).context("Failed to load embeddings from file")?;

    let note_embedding = embeddings
        .iter()
        .find(|e| e.note_path == note_path && e.note_checksum == note_to_checksum(&note))
        .context(format!("Can't find {} in local embeddings. Perhaps the file contents changed and it needs a rebuild?", display_path.yellow()))?
        .to_owned();

    embeddings.try_sort_by_cached_key(|e| {
        Some(-cosine_similarity(&e.embedding, &note_embedding.embedding))
    })?;

    println!();
    println!("Best matches for {}:", display_path.yellow());
    let items = embeddings
        .iter()
        .skip(1)
        .take(50)
        .map(|e| NoteListItem {
            note_path: e.note_path.clone(),
            similarity: cosine_similarity(&e.embedding, &note_embedding.embedding),
        })
        .collect();
    result_selector(items, config, 0)?;

    Ok(())
}

fn load_embeddings(config: &Config) -> anyhow::Result<Vec<Embedding>> {
    let embeddings_buf = fs::read(&config.embedding_path).context("Can't read embeddings file")?;
    let embeddings: Vec<Embedding> = rmp_serde::from_slice(&embeddings_buf)?;
    Ok(embeddings)
}

async fn get_query_embedding(api_key: &str, query: &str) -> anyhow::Result<Vec<f32>> {
    let client = Client::new().with_api_key(api_key);
    let request = CreateEmbeddingRequestArgs::default()
        .model(config::EMBEDDING_MODEL)
        .input(query)
        .build()?;

    let response = client.embeddings().create(request).await?;
    return Ok(response.data.get(0).unwrap().embedding.to_owned());
}

fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    // OpenAI embedding vectors are normalized to [0..1], so it's enough to just compute the dot product
    let mut dot_product = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
    }

    dot_product
}
