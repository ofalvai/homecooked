use owo_colors::OwoColorize;

use crate::{
    common::{collect_notes, note_to_inputs, TOKENIZER},
    config::{self, Config, COST_PER_TOKEN},
};

pub fn calculate_cost(config: &Config) -> anyhow::Result<()> {
    let notes = collect_notes(&config.notes_root);

    println!(
        "Estimating cost of embedding {} notes...",
        notes.len().green()
    );
    println!("Model: {}", config::EMBEDDING_MODEL.blue());
    println!("Cost per token: ${:.7}", COST_PER_TOKEN.green());
    let mut cost = 0.0;
    for note in &notes {
        for input in note_to_inputs(note) {
            let token_count = TOKENIZER.encode_with_special_tokens(&input).len();
            cost += token_count as f64 * COST_PER_TOKEN;
        }
    }

    println!("Total cost: ${:.2}", cost.green());

    Ok(())
}
