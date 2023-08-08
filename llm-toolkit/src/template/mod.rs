use serde::Serialize;
use tinytemplate::TinyTemplate;

#[derive(Serialize)]
pub struct TemplateContext {
    // Main user input to the prompt
    pub input: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("render error: {0}")]
    RenderError(String)
}

pub fn render_prompt(template: &str, ctx: &TemplateContext) -> Result<String, TemplateError> {
    let mut tt = TinyTemplate::new();
    if let Err(err) = tt.add_template("prompt", template) {
        return Err(TemplateError::RenderError(err.to_string()));
    };
    match tt.render("prompt", ctx) {
        Ok(value) => Ok(value),
        Err(err) => Err(TemplateError::RenderError(err.to_string())),
    }
}
