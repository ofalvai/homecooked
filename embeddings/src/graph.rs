use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use regex::Regex;

use crate::common::Note;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct PathPair(PathBuf, PathBuf);

impl PathPair {
    pub fn new(path_a: &PathBuf, path_b: &PathBuf) -> Self {
        if path_a < path_b {
            PathPair(path_a.clone(), path_b.clone())
        } else {
            PathPair(path_b.clone(), path_a.clone())
        }
    }
}

pub struct LinkGraph {
    adj: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl LinkGraph {
    pub fn new() -> Self {
        Self { adj: HashMap::new() }
    }

    pub fn add_link(&mut self, path_a: &Path, path_b: &Path) {
        if path_a == path_b {
            return;
        }
        self.adj.entry(path_a.to_path_buf())
            .or_insert_with(HashSet::new)
            .insert(path_b.to_path_buf());
        self.adj.entry(path_b.to_path_buf())
            .or_insert_with(HashSet::new)
            .insert(path_a.to_path_buf());
    }

    pub fn all_linked_pairs(&self) -> HashSet<PathPair> {
        let mut pairs = HashSet::new();
        for (path_a, neighbors) in &self.adj {
            for path_b in neighbors {
                if path_a < path_b { // each link appears twice, so only add one direction
                    pairs.insert(PathPair(path_a.clone(), path_b.clone()));
                }
            }
        }
        pairs
    }

    pub fn from_notes(notes: &[Note], exclude_prefixes: &[PathBuf]) -> Self {
        let mut graph = Self::new();

        let all_paths: HashSet<PathBuf> = notes.iter()
            .filter(|n| !is_excluded(&n.path, exclude_prefixes))
            .map(|n| n.path.clone())
            .collect();

        let title_to_path: HashMap<String, PathBuf> = notes.iter()
            .filter(|n| !is_excluded(&n.path, exclude_prefixes))
            .map(|n| (n.title.clone(), n.path.clone()))
            .collect();

        for note in notes {
            if is_excluded(&note.path, exclude_prefixes) {
                continue;
            }
            let links = parse_wikilinks(&note.text_content);
            for link_target in links {
                if let Some(target_path) = resolve_link(&link_target, &all_paths, &title_to_path) {
                    graph.add_link(&note.path, &target_path);
                }
            }
        }

        graph
    }
}

fn is_excluded(path: &Path, exclude_prefixes: &[PathBuf]) -> bool {
    for prefix in exclude_prefixes {
        if path.starts_with(prefix) {
            return true;
        }
    }
    false
}

fn resolve_link(
    link_text: &str,
    all_paths: &HashSet<PathBuf>,
    title_to_path: &HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    let link_path = PathBuf::from(link_text);

    if all_paths.contains(&link_path) {
        return Some(link_path);
    }

    title_to_path.get(link_text).cloned()
}

pub fn parse_wikilinks(content: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_wikilink() {
        assert_eq!(parse_wikilinks("See [[Meeting Notes]] for details"), vec!["Meeting Notes"]);
    }

    #[test]
    fn wikilink_with_alias() {
        assert_eq!(parse_wikilinks("Check [[vault/projects/this doc.md|Project Alpha]]"), vec!["vault/projects/this doc.md"]);
    }

    #[test]
    fn multiple_wikilinks() {
        let links = parse_wikilinks("Related: [[A]] and [[B]]");
        assert_eq!(links, vec!["A", "B"]);
    }

    #[test]
    fn no_wikilinks() {
        assert!(parse_wikilinks("No links here").is_empty());
    }

    #[test]
    fn self_link_ignored() {
        let mut graph = LinkGraph::new();
        graph.add_link(Path::new("a.md"), Path::new("a.md"));
        assert!(graph.all_linked_pairs().is_empty());
    }

    #[test]
    fn all_linked_pairs_returns_hashset() {
        let mut graph = LinkGraph::new();
        graph.add_link(Path::new("a.md"), Path::new("b.md"));
        graph.add_link(Path::new("a.md"), Path::new("c.md"));
        let pairs = graph.all_linked_pairs();
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&PathPair::new(&PathBuf::from("a.md"), &PathBuf::from("b.md"))));
        assert!(pairs.contains(&PathPair::new(&PathBuf::from("a.md"), &PathBuf::from("c.md"))));
    }

    #[test]
    fn path_pair_canonical_ordering() {
        let a = PathBuf::from("a.md");
        let b = PathBuf::from("b.md");
        let pair_ab = PathPair::new(&a, &b);
        let pair_ba = PathPair::new(&b, &a);
        assert_eq!(pair_ab, pair_ba);
    }

    #[test]
    fn is_excluded_directory_prefix() {
        let path = PathBuf::from("/Notes/Archive/2024-01.md");
        let excludes = vec![PathBuf::from("/Notes/Archive")];
        assert!(is_excluded(&path, &excludes));
    }

    #[test]
    fn is_excluded_does_not_match_similar() {
        let path = PathBuf::from("/Notes/Archive-test/2024-01.md");
        let excludes = vec![PathBuf::from("/Notes/Archive")];
        assert!(!is_excluded(&path, &excludes));
    }

    #[test]
    fn resolve_link_exact_path() {
        let all_paths = vec![
            PathBuf::from("Projects/App.md"),
            PathBuf::from("Archive/Old.md"),
        ].into_iter().collect();

        let title_to_path = HashMap::new();

        let result = resolve_link("Projects/App.md", &all_paths, &title_to_path);
        assert_eq!(result, Some(PathBuf::from("Projects/App.md")));
    }

    #[test]
    fn resolve_link_title_match() {
        let all_paths = vec![
            PathBuf::from("Projects/App.md"),
            PathBuf::from("Archive/Old.md"),
        ].into_iter().collect();

        let mut title_to_path = HashMap::new();
        title_to_path.insert("App".to_string(), PathBuf::from("Projects/App.md"));

        let result = resolve_link("App", &all_paths, &title_to_path);
        assert_eq!(result, Some(PathBuf::from("Projects/App.md")));
    }

    #[test]
    fn resolve_link_not_found() {
        let all_paths: HashSet<PathBuf> = vec![
            PathBuf::from("Projects/App.md"),
        ].into_iter().collect();

        let title_to_path: HashMap<String, PathBuf> = HashMap::new();

        let result = resolve_link("Nonexistent", &all_paths, &title_to_path);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_link_path_before_title() {
        let all_paths = vec![
            PathBuf::from("Work/Meeting Notes.md"),
        ].into_iter().collect();

        let mut title_to_path = HashMap::new();
        title_to_path.insert("Meeting Notes".to_string(), PathBuf::from("Personal/Meeting Notes.md"));

        let result = resolve_link("Work/Meeting Notes.md", &all_paths, &title_to_path);
        assert_eq!(result, Some(PathBuf::from("Work/Meeting Notes.md")));
    }
}
