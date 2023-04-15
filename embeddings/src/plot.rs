use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use plotly::{
    common::{Marker, Mode, Title},
    Layout, Plot, Scatter,
};

use crate::{
    common::load_embeddings,
    config::{Config, EMBEDDING_DIM},
    types::Embedding,
};

struct Note {
    x: f32,
    y: f32,
    path: PathBuf,
}

pub fn plot(config: &Config) -> anyhow::Result<()> {
    println!("Loading embeddings...");
    let embeddings = load_embeddings(config)?;

    println!("Computing 2D representation using t-SNE...this may take a while...");
    let embeddings_2d: Vec<Note> = t_sne(&embeddings)
        .into_iter()
        .enumerate()
        .map(|(i, vec)| {
            let path = &embeddings
                .get(i)
                .context(format!("Failed to get embedding at index {}", i))?
                .note_path;
            Ok(Note {
                x: vec[0],
                y: vec[1],
                path: path.clone(),
            })
        })
        .collect::<anyhow::Result<Vec<Note>>>()
        .context("Failed to create 2D representation of embeddings")?;
    println!("Done!");

    show_plot(&embeddings_2d, &config.plot_colors);
    Ok(())
}

fn t_sne(embeddings: &[Embedding]) -> Vec<[f32; 2]> {
    let vectors: Vec<[f32; EMBEDDING_DIM]> = embeddings
        .iter()
        .map(|e| e.embedding.as_slice().try_into().unwrap())
        .collect();

    bhtsne::tSNE::new(&vectors)
        .embedding_dim(2)
        .epochs(2000)
        .perplexity(20.0)
        .learning_rate(10.0)
        .barnes_hut(0.5, |sample_a, sample_b| {
            sample_a
                .iter()
                .zip(sample_b.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt()
        })
        .embedding()
        .chunks(2)
        .map(|chunk| [chunk[0], chunk[1]])
        .collect()
}

fn show_plot(notes: &[Note], color_map: &HashMap<String, String>) {
    let ((x, y), label): ((Vec<f32>, Vec<f32>), Vec<String>) = notes
        .iter()
        .map(|n| ((n.x, n.y), n.path.to_string_lossy().into_owned()))
        .unzip();

    let color: Vec<String> = notes
        .iter()
        .map(|n| path_to_color(&n.path, color_map))
        .collect();

    let mut plot = Plot::new();
    let layout = Layout::new()
        .title(Title::new("Note similarity (t-SNE)"))
        .auto_size(true)
        .height(1000);
    plot.set_layout(layout);
    let trace = Scatter::new(x, y)
        .mode(Mode::Markers)
        .text_array(label)
        .name("Notes")
        .marker(Marker::new().color_array(color));
    plot.add_trace(trace);
    plot.show();
}

fn path_to_color(path: &Path, color_map: &HashMap<String, String>) -> String {
    color_map
        .iter()
        .find_map(|(k, v)| {
            if path.starts_with(k) {
                Some(v.to_owned())
            } else {
                None
            }
        })
        .unwrap_or(String::from("gray"))
}
