use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Embedding {
    pub note_path: PathBuf,
    pub note_checksum: u32,
    pub embedding: Vec<f32>,
}
