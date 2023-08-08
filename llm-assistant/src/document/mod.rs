pub mod loader;

pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: DocumentMetadata,
}

pub struct DocumentMetadata {
    pub name: String,
    pub kind: String,
    pub comment: Option<String>,
}
