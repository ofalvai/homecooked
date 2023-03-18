use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ignore::Walk;
use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;

pub fn run_clean(root: &str, dry_run: bool) -> anyhow::Result<()> {
    let root_path = Path::new(root).canonicalize().unwrap();
    let (notes, other_files) = collect_files(&root_path);
    let notes_set: HashSet<PathBuf> = notes.into_iter().collect();
    let other_files_set: HashSet<PathBuf> = other_files.into_iter().collect();

    let mut referenced_files = vec![];

    for note in &notes_set {
        let mut files = collect_referenced_files(&note).with_context(|| {
            format!("Failed to collect referenced files in {}", &note.display())
        })?;
        referenced_files.append(&mut files);
    }

    if referenced_files.is_empty() {
        println!("No file references found in notes");
        return Ok(());
    }

    let referenced_files_set: HashSet<PathBuf> = referenced_files.into_iter().collect();

    println!("Referenced files from notes:");
    for file in &referenced_files_set {
        println!("- {}", file.strip_prefix(&root_path).unwrap().display())
    }
    println!();

    let unreferenced_files_set = other_files_set.difference(&referenced_files_set);
    println!("Unreferenced files found:");
    for file in unreferenced_files_set {
        print!("- {}", file.strip_prefix(&root_path).unwrap().display());

        if !dry_run {
            fs::remove_file(file)
                .with_context(|| format!("Failed to remove {}", file.display()))?;
            print!(" => deleted")
        }
        println!();
    }

    Ok(())
}

// First item is list of Markdown files, the second is the rest
fn collect_files(root: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>) {
    Walk::new(root)
        .filter_map(|result| {
            let entry = result.expect("Error iterating over files");
            let metadata = entry.metadata().unwrap();
            match metadata.is_dir() {
                true => None,
                false => Some(entry.path().to_path_buf()),
            }
        })
        .partition(|entry| {
            let extension = entry
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap();
            extension == "md"
        })
}

fn collect_referenced_files(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let content = fs::read_to_string(path).context("Failed to read file contents")?;

    let parser = Parser::new_ext(&content, Options::empty());
    let files: Vec<PathBuf> = parser
        .filter_map(|e| match e {
            Event::Start(Tag::Image(_linktype, dest, _title)) => Some(dest.to_string()),
            Event::Start(Tag::Link(_linktype, dest, _title)) => Some(dest.to_string()),
            _ => None,
        })
        .filter(|dest| !is_web_link(dest))
        .filter(|dest| !dest.ends_with(".md"))
        .map(|dest| {
            let parent = path.parent().expect("Note should have a parent folder");
            let dest_decoded = percent_decode_str(&dest).decode_utf8_lossy();
            let joined = parent.join(dest_decoded.as_ref());
            fs::canonicalize(joined.clone())
                .expect(format!("Problem with path: {}", &joined.display()).as_str())
        })
        .collect();

    Ok(files)
}

fn is_web_link(link: &str) -> bool {
    let re = Regex::new(r"(?i)^[a-z]+://.*").unwrap();
    re.is_match(link)
}
