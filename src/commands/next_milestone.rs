use anyhow::{Result, bail};
use std::path::Path;

use crate::recipe;
use crate::state;
use crate::ui;

pub fn run(workspace_hint: &Path) -> Result<()> {
    let mut state = state::load_from_workspace(workspace_hint)?;
    let recipe = recipe::load_from_path(&state.recipe_path)?;
    let current = recipe::resolve_initial_milestone(&recipe, Some(&state.milestone_id))?;
    let current_index = recipe::milestone_index(&recipe, &state.milestone_id)?;

    if state.verified_milestone_id.as_deref() != Some(current.id.as_str()) {
        bail!(
            "milestone {} is not verified yet; run the {} first",
            current.id,
            ui::reference("skill", "primer-verify")
        );
    }

    let Some(next) = recipe.milestones.get(current_index + 1) else {
        ui::section("Primer next milestone");
        println!();
        ui::success("Recipe complete");
        println!(
            "The final milestone {} is already verified. No state changes were made.",
            ui::code(&current.id)
        );
        return Ok(());
    };

    state.milestone_id = next.id.clone();
    state.verified_milestone_id = None;
    state::write(&state)?;

    let spec_path = recipe
        .path
        .join("milestones")
        .join(&next.id)
        .join("spec.md");
    let explanation_path = recipe
        .path
        .join("milestones")
        .join(&next.id)
        .join("explanation.md");

    ui::section("Primer next milestone");
    println!();
    ui::success(&format!("Advanced to {}", next.id));
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Previous milestone".to_string(),
            value: format!("{} ({})", current.id, current.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{} ({})", next.id, next.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: state.track.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "State file".to_string(),
            value: state.context_path.display().to_string(),
            value_color: Some(comfy_table::Color::DarkGrey),
        },
    ]);
    println!();
    ui::section("Next");
    ui::numbered_steps(&[
        format!("Read {}", ui::code(spec_path.display().to_string())),
        format!(
            "Read {} for the deep-dive explanation",
            ui::code(explanation_path.display().to_string())
        ),
        format!(
            "Run the {} to work on {}",
            ui::reference("skill", "primer-build"),
            ui::code(&next.id)
        ),
    ]);

    Ok(())
}
