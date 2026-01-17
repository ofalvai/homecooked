use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use owo_colors::OwoColorize;
use rayon::prelude::*;

use crate::common::{collect_notes, load_embeddings};
use crate::config::Config;
use crate::graph::{LinkGraph, PathPair};
use crate::search::cosine_similarity;
use crate::prompt::unlinked_selector;

pub struct UnlinkedPair {
    pub path_a: PathBuf,
    pub path_b: PathBuf,
    pub similarity: f32,
}

fn is_excluded(path: &Path, exclude_prefixes: &[PathBuf]) -> bool {
    for prefix in exclude_prefixes {
        if path.starts_with(prefix) {
            return true;
        }
    }
    false
}

pub async fn find_unlinked(config: &Config, threshold: f32, exclude_patterns: &[String]) -> anyhow::Result<Vec<UnlinkedPair>> {
    let load_start = Instant::now();
    let embeddings = load_embeddings(config)?;
    let notes = collect_notes(&config.notes_root);
    println!("Loaded {} embeddings and {} notes in {:?}", embeddings.len(), notes.len(), load_start.elapsed().green());

    let filter_start = Instant::now();
    let exclude_prefixes: Vec<PathBuf> = exclude_patterns.iter().map(PathBuf::from).collect();
    let original_count = embeddings.len();
    let filtered_embeddings: Vec<_> = embeddings.into_iter()
        .filter(|e| !is_excluded(&e.note_path, &exclude_prefixes))
        .collect();
    println!("Filtered to {} embeddings (excluded {} by pattern)", filtered_embeddings.len(), original_count - filtered_embeddings.len());
    println!("Filtering took {:?}", filter_start.elapsed().green());

    let graph_start = Instant::now();
    let graph = LinkGraph::from_notes(&notes, &exclude_prefixes);
    println!("Built link graph in {:?}", graph_start.elapsed().green());

    let linked_set_start = Instant::now();
    let linked_pairs = graph.all_linked_pairs();
    println!("Pre-computed {} linked pairs in {:?}", linked_pairs.len(), linked_set_start.elapsed().green());

    let compare_start = Instant::now();
    let total_comparisons = (filtered_embeddings.len() * (filtered_embeddings.len() - 1)) / 2;
    let embeddings_len = filtered_embeddings.len();
    println!("Comparing {} pairs...", total_comparisons);

    // Arc enables safe sharing across Rayon's parallel worker threads.
    // Cloning Arc is cheap (just increments atomic refcount) vs cloning the actual data.
    // Each worker thread clones the Arc to get its own reference handle.
    let embeddings_arc = Arc::new(filtered_embeddings);
    let linked_pairs_arc = Arc::new(linked_pairs);

    let all_pairs: Vec<UnlinkedPair> = (0..embeddings_len - 1)
        .into_par_iter()
        .flat_map(|i| {
            // Each Rayon worker needs its own Arc reference.
            // Arc::clone is O(1) - just atomic increment, no data copy.
            let embeddings_ref = Arc::clone(&embeddings_arc);
            let linked_ref = Arc::clone(&linked_pairs_arc);

            let mut local_pairs = Vec::new();

            for j in (i + 1)..embeddings_ref.len() {
                let path_a = &embeddings_ref[i].note_path;
                let path_b = &embeddings_ref[j].note_path;

                let pair = PathPair::new(path_a, path_b);
                if linked_ref.contains(&pair) {
                    continue;
                }

                let similarity = cosine_similarity(&embeddings_ref[i].embedding, &embeddings_ref[j].embedding);
                if similarity >= threshold {
                    local_pairs.push(UnlinkedPair {
                        path_a: path_a.clone(),
                        path_b: path_b.clone(),
                        similarity,
                    });
                }
            }

            local_pairs
        })
        .collect();

    println!("Compared {} pairs in {:?}", total_comparisons, compare_start.elapsed().green());
    println!("Found {} unlinked similar pairs", all_pairs.len());

    let mut pairs = all_pairs;
    pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    Ok(pairs)
}

pub async fn handle_unlinked(config: &Config, output: Option<&str>, threshold: u8, exclude_patterns: &[String]) -> anyhow::Result<()> {
    let threshold_val = threshold as f32 / 100.0;
    let pairs = find_unlinked(config, threshold_val, exclude_patterns).await?;

    if pairs.is_empty() {
        println!("No unlinked similar note pairs found above {}% threshold", threshold);
        return Ok(());
    }

    if let Some(output_path) = output {
        write_unlinked_markdown(output_path, &pairs, threshold)?;
        println!("Wrote {} suggestions to {}", pairs.len(), output_path);
    } else {
        unlinked_selector(pairs, config)?;
    }

    Ok(())
}

fn write_unlinked_markdown(path: &str, pairs: &[UnlinkedPair], threshold: u8) -> anyhow::Result<()> {
    let mut content = format!("# Unlinked but similar note pairs\n\nFound {} unlinked similar note pairs (similarity > {}%)\n\n", pairs.len(), threshold);

    for pair in pairs {
        let title_a = pair.path_a.file_stem().unwrap_or_default().to_string_lossy();
        let title_b = pair.path_b.file_stem().unwrap_or_default().to_string_lossy();
        let similarity_pct = (pair.similarity * 100.0).round() as u8;

        let display_a = if title_a == title_b {
            pair.path_a.display().to_string()
        } else {
            title_a.to_string()
        };
        let display_b = if title_a == title_b {
            pair.path_b.display().to_string()
        } else {
            title_b.to_string()
        };

        content.push_str(&format!("- [[{}]] <-> [[{}]] ({}%)\n", display_a, display_b, similarity_pct));
    }

    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_excluded_excludes_directory_prefix() {
        let path = PathBuf::from("/Notes/Archive/2024-01.md");
        let excludes = vec![PathBuf::from("/Notes/Archive")];
        assert!(is_excluded(&path, &excludes));
    }

    #[test]
    fn test_is_excluded_does_not_match_similar_directory() {
        let path = PathBuf::from("/Notes/Archive-test/2024-01.md");
        let excludes = vec![PathBuf::from("/Notes/Archive")];
        assert!(!is_excluded(&path, &excludes));
    }

    #[test]
    fn test_is_excluded_excludes_exact_file() {
        let path = PathBuf::from("/Notes/Templates/daily.md");
        let excludes = vec![PathBuf::from("/Notes/Templates/daily.md")];
        assert!(is_excluded(&path, &excludes));
    }

    #[test]
    fn test_is_excluded_does_not_match_file_in_directory() {
        let path = PathBuf::from("/Notes/foo/daily.md");
        let excludes = vec![PathBuf::from("/Notes/Templates/daily.md")];
        assert!(!is_excluded(&path, &excludes));
    }

    #[test]
    fn test_is_excluded_handles_multiple_patterns() {
        let excludes = vec![
            PathBuf::from("/Notes/Archive"),
            PathBuf::from("/Notes/Templates"),
        ];

        assert!(is_excluded(&PathBuf::from("/Notes/Archive/old.md"), &excludes));
        assert!(is_excluded(&PathBuf::from("/Notes/Templates/template.md"), &excludes));
        assert!(!is_excluded(&PathBuf::from("/Notes/Projects/foo.md"), &excludes));
    }

    #[test]
    fn test_is_excluded_returns_false_for_empty_patterns() {
        let path = PathBuf::from("/Notes/foo.md");
        let excludes = vec![];
        assert!(!is_excluded(&path, &excludes));
    }

    #[test]
    fn test_is_excluded_nested_directory() {
        let path = PathBuf::from("/Notes/Journal/2024/01/16.md");
        let excludes = vec![PathBuf::from("/Notes/Journal/2024")];
        assert!(is_excluded(&path, &excludes));
    }
}
