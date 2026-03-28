use anyhow::{Context, Result, bail};
use comfy_table::Color;
use std::fs;
use std::path::Path;

use crate::state;
use crate::ui;
use crate::workflow;

pub fn run(workspace_hint: &Path) -> Result<()> {
    let state = state::load_from_workspace(workspace_hint)?;
    let workflow = workflow::load(&state.source)?;
    let milestone = workflow::resolve_initial_milestone(&workflow, Some(&state.milestone_id))?;

    let explanation_path = workflow
        .path
        .join("milestones")
        .join(&milestone.id)
        .join("explanation.md");

    if !explanation_path.is_file() {
        bail!(
            "milestone explanation not found at {}",
            explanation_path.display()
        );
    }

    let explanation = fs::read_to_string(&explanation_path)
        .with_context(|| format!("failed to read {}", explanation_path.display()))?;

    ui::section("Primer explain");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: state.source.kind.label().to_string(),
            value: state.source.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{} ({})", milestone.id, milestone.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Explanation file".to_string(),
            value: explanation_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);
    println!();
    ui::print_markdown(explanation.trim_end());

    Ok(())
}
