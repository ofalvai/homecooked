use log::debug;
use youtube_transcript::YoutubeBuilder;

pub struct Transcript {
    pub text: String,
}

pub async fn fetch_transcript(url: String) -> Result<Transcript, super::LoadError> {
    let result = match YoutubeBuilder::default().build().transcript(&url).await {
        Ok(t) => t,
        Err(e) => return Err(super::LoadError::ProcessingError(e.to_string())),
    };

    match result
        .into_iter()
        .map(|t| {
            let mut cleaned = String::new();
            html_escape::decode_html_entities_to_string(t.text.replace("\n", " "), &mut cleaned);
            cleaned
        })
        .reduce(|acc, e| format!("{} {}", acc, e))
        .ok_or(super::LoadError::ProcessingError(
            "no transcript chunks returned".to_string(),
        )) {
        Ok(t) => {
            debug!("Transcript length: {} chars", t.chars().count());
            return Ok(Transcript { text: t });
        }
        Err(e) => return Err(e),
    };
}
