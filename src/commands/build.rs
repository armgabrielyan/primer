use anyhow::{Context, Result, bail};
use comfy_table::Color;
use std::fs;
use std::path::Path;

use crate::recipe;
use crate::state;
use crate::ui;

pub fn run(primer_root: &Path, workspace_hint: &Path) -> Result<()> {
    let state = state::load_from_workspace(workspace_hint)?;
    let recipe = recipe::load_by_id(primer_root, &state.recipe_id)?;
    let milestone = recipe::resolve_initial_milestone(&recipe, Some(&state.milestone_id))?;

    if state.recipe_path != recipe.path {
        bail!(
            "workspace state points to {}, but resolved recipe is {}",
            state.recipe_path.display(),
            recipe.path.display()
        );
    }

    let milestone_dir = recipe.path.join("milestones").join(&milestone.id);
    let spec_path = milestone_dir.join("spec.md");
    let agent_path = milestone_dir.join("agent.md");

    if !spec_path.is_file() {
        bail!("milestone spec not found at {}", spec_path.display());
    }
    if !agent_path.is_file() {
        bail!(
            "milestone agent instructions not found at {}",
            agent_path.display()
        );
    }

    let spec = fs::read_to_string(&spec_path)
        .with_context(|| format!("failed to read {}", spec_path.display()))?;
    let agent = fs::read_to_string(&agent_path)
        .with_context(|| format!("failed to read {}", agent_path.display()))?;
    let track_instructions = extract_track_section(&agent, &state.track)?;

    ui::section("Primer build");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Recipe".to_string(),
            value: recipe.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{} ({})", milestone.id, milestone.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: state.track.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workspace".to_string(),
            value: state.workspace_root.display().to_string(),
            value_color: Some(Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Spec file".to_string(),
            value: spec_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
        ui::KeyValueRow {
            key: "Agent file".to_string(),
            value: agent_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);

    println!();
    ui::section("Milestone spec");
    println!();
    ui::print_markdown(spec.trim_end());

    println!();
    ui::section("Track guidance");
    println!();
    ui::print_markdown(track_instructions.trim_end());

    println!();
    ui::section("Next");
    ui::numbered_steps(&[
        format!(
            "Implement only {} in {}",
            ui::code(&milestone.id),
            ui::code(state.workspace_root.display().to_string())
        ),
        format!(
            "Run the {} when the milestone is complete",
            ui::reference("skill", "primer-check")
        ),
    ]);

    Ok(())
}

fn extract_track_section<'a>(agent_markdown: &'a str, track: &str) -> Result<&'a str> {
    let heading = match track {
        "learner" => "## Learner Track",
        "builder" => "## Builder Track",
        other => bail!("unsupported track '{}' in workspace state", other),
    };

    let start = agent_markdown
        .find(heading)
        .ok_or_else(|| anyhow::anyhow!("missing '{}' section in agent instructions", heading))?;
    let rest = &agent_markdown[start..];
    let end = rest
        .match_indices("\n## ")
        .map(|(index, _)| index)
        .find(|index| *index > 0)
        .unwrap_or(rest.len());
    Ok(rest[..end].trim())
}
