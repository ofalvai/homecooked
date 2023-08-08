use anyhow::Context;
use serde::Serialize;
use tinytemplate::TinyTemplate;

#[derive(Serialize)]
pub struct TemplateContext {
    // Main user input to the prompt
    pub input: String,
}

pub fn render_prompt(template: &str, ctx: &TemplateContext) -> anyhow::Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("prompt", template)?;
    tt.render("prompt", ctx).context("template error")
}
