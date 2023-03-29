use std::time::Instant;

use anyhow::Context;
use owo_colors::OwoColorize;

use async_openai::{types::CreateEmbeddingRequestArgs, Client};
use try_partialord::TrySort;

use crate::{
    config::{self, Config},
    types::Embedding,
};

pub async fn query(config: &Config, query: &str) -> anyhow::Result<()> {
    println!("Embedding query...");
    let embedding_start = Instant::now();
    let query_embedding = get_query_embedding(&config.api_key, query).await?;
    let embedding_duration = embedding_start.elapsed();
    println!("Done");

    let parse_start = Instant::now();

    let embeddings_buf = std::fs::read(&config.embedding_path).context("Can't read embeddings file")?;
    let mut embeddings: Vec<Embedding> = rmp_serde::from_slice(&embeddings_buf)?;
    let parse_duration = parse_start.elapsed();

    let sort_start = Instant::now();
    embeddings
        .try_sort_by_cached_key(|e| Some(-cosine_similarity(&e.embedding, &query_embedding)))?;
    let sort_duration = sort_start.elapsed();

    println!();
    println!("Best matches for {}:", query.yellow());
    for embedding in &embeddings[0..10] {
        let similarity = cosine_similarity(&embedding.embedding, &query_embedding);
        println!(
            "{}: {}",
            embedding.note_path.to_string_lossy().bold(),
            format!("{:.0}%", similarity * 100.0).green()
        );
    }

    println!();
    println!("Query embedding time: {:?}", embedding_duration.green());
    println!("Parse time: {:?}", parse_duration.green());
    println!("Sort time: {:?}", sort_duration.green());
    println!("Note count: {}", embeddings.len().to_string().green());
    Ok(())
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
    let mut dot_product = 0.0;
    let mut a_norm = 0.0;
    let mut b_norm = 0.0;

    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        a_norm += a[i] * a[i];
        b_norm += b[i] * b[i];
    }

    dot_product / (a_norm * b_norm).sqrt()
}
