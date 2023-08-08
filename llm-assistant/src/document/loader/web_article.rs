use article_scraper::Readability;

use super::LoadError;

pub struct WebArticleLoader {}

impl WebArticleLoader {
    pub async fn load(&self, url: &str) -> Result<String, LoadError> {
        let response = match reqwest::get(url).await {
            Ok(response) => response,
            Err(err) => return Err(LoadError::NetworkRequestError(err.to_string())),
        };

        let html = match response.text().await {
            Ok(response) => response,
            Err(err) => return Err(LoadError::NetworkRequestError(err.to_string())),
        };
        let extracted_content = match Readability::extract(&html, None).await {
            Ok(content) => content,
            Err(err) => return Err(LoadError::ProcessingError(err.to_string())),
        };

        Ok(extracted_content)
    }
}
