use std::{fs, io, path::PathBuf};

use anyhow::Context;
use obsidian_export::Context as ObsidianContext;
use obsidian_export::{serde_yaml, Exporter, MarkdownEvents, PostprocessorResult, WalkOptions};

// TODO:
// - Handle line breaks?
// - PR for configurable warnings

const FRONTMATTER_KEY_TAGS: &str = "tags";
const FRONTMATTER_TAG_INCLUDE: &str = "public";

pub fn run_export(root: &str, dest: &str) -> anyhow::Result<()> {
    let root = PathBuf::from(root);
    let destination = PathBuf::from(dest);
    let walk_options = WalkOptions::new();

    ensure_destination_dir(&destination).context("Failed to prepare destination dir")?;

    let mut exporter = Exporter::new(root, destination);
    exporter.walk_options(walk_options);

    let filter_processor = create_frontmatter_filter();
    exporter.add_postprocessor(&filter_processor);

    let title_processor = create_title_appender();
    exporter.add_postprocessor(&title_processor);

    exporter.run()?;
    Ok(())
}

fn ensure_destination_dir(destination: &PathBuf) -> io::Result<()> {
    let exists = destination.try_exists()?;
    if !exists {
        fs::create_dir_all(destination)?;
    }
    Ok(())
}

fn create_frontmatter_filter(
) -> impl Fn(&mut ObsidianContext, &mut MarkdownEvents) -> PostprocessorResult {
    move |context, _events| match context.frontmatter.get(FRONTMATTER_KEY_TAGS) {
        Some(serde_yaml::Value::Sequence(tags)) => {
            if tags.contains(&serde_yaml::to_value(FRONTMATTER_TAG_INCLUDE).unwrap()) {
                // Remove all tags, they are not public-facing
                context.frontmatter.remove(FRONTMATTER_KEY_TAGS);
                PostprocessorResult::Continue
            } else {
                PostprocessorResult::StopAndSkipNote
            }
        }
        _ => PostprocessorResult::StopAndSkipNote,
    }
}

fn create_title_appender(
) -> impl Fn(&mut ObsidianContext, &mut MarkdownEvents) -> PostprocessorResult {
    |context, _events| {
        let k = serde_yaml::to_value("title").unwrap();
        let v = serde_yaml::to_value(context.destination.file_stem().unwrap().to_str()).unwrap();
        context.frontmatter.insert(k, v);
        PostprocessorResult::Continue
    }
}
