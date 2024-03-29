use serde::Serialize;
use tinytemplate::{format_unescaped, TinyTemplate};

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
    RenderError(String),
}

// Render a prompt template with the given context.
// Template syntax is [TinyTemplate](https://docs.rs/tinytemplate/latest/tinytemplate/syntax/index.html).
pub fn render_prompt(template: &str, ctx: &TemplateContext) -> Result<String, TemplateError> {
    render(template, ctx, "prompt")
}

// Render a prompt template with a generic context object.
// Template syntax is [TinyTemplate](https://docs.rs/tinytemplate/latest/tinytemplate/syntax/index.html).
pub fn render<C>(template: &str, ctx: C, name: &str) -> Result<String, TemplateError>
where
    C: Serialize,
{
    let mut tt = TinyTemplate::new();

    if let Err(err) = tt.add_template(name, template) {
        return Err(TemplateError::RenderError(err.to_string()));
    };
    tt.set_default_formatter(&format_unescaped);
    match tt.render(name, &ctx) {
        Ok(value) => Ok(value),
        Err(err) => Err(TemplateError::RenderError(err.to_string())),
    }
}
