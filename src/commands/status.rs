use anyhow::Result;
use comfy_table::Color;
use std::path::Path;

use crate::recipe;
use crate::state;
use crate::ui;

pub fn run(workspace_hint: &Path) -> Result<()> {
    let state = state::load_from_workspace(workspace_hint)?;
    let recipe = recipe::load_from_path(&state.recipe_path)?;
    let current = recipe::resolve_initial_milestone(&recipe, Some(&state.milestone_id))?;
    let current_index = recipe::milestone_index(&recipe, &state.milestone_id)?;
    let verified = state.verified_milestone_id.as_deref() == Some(state.milestone_id.as_str());
    let next = recipe.milestones.get(current_index + 1);

    ui::section("Primer status");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Recipe".to_string(),
            value: state.recipe_id,
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: state.track,
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Stack".to_string(),
            value: state.stack_id,
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workspace".to_string(),
            value: state.workspace_root.display().to_string(),
            value_color: Some(Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{} ({})", current.id, current.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Verified".to_string(),
            value: if verified {
                "yes".to_string()
            } else {
                "no".to_string()
            },
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Progress".to_string(),
            value: format!("{}/{}", current_index + 1, recipe.milestones.len()),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Next milestone".to_string(),
            value: next
                .map(|m| format!("{} ({})", m.id, m.title))
                .unwrap_or_else(|| "complete".to_string()),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "State file".to_string(),
            value: state.context_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);

    println!();
    ui::section("Next");
    if verified {
        if let Some(next) = next {
            ui::numbered_steps(&[
                format!(
                    "Run the {} to advance to {}",
                    ui::reference("skill", "primer-next-milestone"),
                    ui::code(&next.id)
                ),
                format!(
                    "Read the next milestone spec and explanation for {}",
                    ui::code(&next.id)
                ),
            ]);
        } else {
            ui::numbered_steps(&["Recipe is complete.".to_string()]);
        }
    } else {
        ui::numbered_steps(&[
            format!(
                "Run the {} to work on {}",
                ui::reference("skill", "primer-build"),
                ui::code(&current.id)
            ),
            format!(
                "Run the {} when the milestone is complete",
                ui::reference("skill", "primer-verify")
            ),
        ]);
    }

    Ok(())
}
