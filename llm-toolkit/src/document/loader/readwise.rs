use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const LIST_URL: &str = "https://readwise.io/api/v3/list";

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    New,
    Later,
    Shortlist,
    Archive,
    Feed
}

#[derive(Debug, Deserialize)]
pub struct Document {
    pub id: String,
    pub url: String,
    pub source_url: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub author: String,
    pub site_name: Option<String>,
    pub word_count: Option<u32>,
    pub reading_progress: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentListResponse {
    results: Vec<Document>,
    next_page_cursor: Option<String>,
}

pub struct ReadwiseClient {
    token: String,
}

impl ToString for Location {
    fn to_string(&self) -> String {
        match self {
            Location::New => "new".to_string(),
            Location::Later => "later".to_string(),
            Location::Shortlist => "shortlist".to_string(),
            Location::Archive => "archive".to_string(),
            Location::Feed => "feed".to_string(),
        }
    }
}

impl ReadwiseClient {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl ReadwiseClient {
    pub async fn fetch_documents(&self, updated_after: Option<String>, location: Option<Location>) -> Result<Vec<Document>, reqwest::Error> {
        let mut documents = Vec::new();
        let mut next_page_cursor: Option<String> = None;
    
        loop {
            let mut query_params = HashMap::new();
            if let Some(cursor) = next_page_cursor {
                query_params.insert("pageCursor", cursor);
            }
            if let Some(after) = &updated_after {
                query_params.insert("updatedAfter", after.into());
            }
            if let Some(loc) = &location {
                query_params.insert("location", loc.to_string());
            }
    
            let response = reqwest::Client::new()
                .get(LIST_URL)
                .query(&query_params)
                .header(reqwest::header::AUTHORIZATION, format!("Token {}", self.token))
                .send()
                .await?;

            debug!("GET list: {}", response.status());
            // debug!("Response body: {}", response.text().await?);

            let json = response
                .json::<DocumentListResponse>()
                .await?;
    
            documents.extend(json.results);
    
            next_page_cursor = json.next_page_cursor;
            if next_page_cursor.is_none() {
                break;
            }
        }
        Ok(documents)
    }
    
}

