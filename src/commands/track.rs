use anyhow::Result;
use comfy_table::Color;
use std::path::Path;

use crate::cli::TrackArgs;
use crate::state;
use crate::ui;
use crate::workflow;

pub fn run(workspace_hint: &Path, args: TrackArgs) -> Result<()> {
    let mut state = state::load_from_workspace(workspace_hint)?;
    let workflow = workflow::load(&state.source)?;
    let milestone = workflow::resolve_initial_milestone(&workflow, Some(&state.milestone_id))?;
    let previous_track = state.track.clone();
    let next_track = args.track.as_str();
    let changed = previous_track != next_track;
    let current_milestone_verified =
        state.verified_milestone_id.as_deref() == Some(state.milestone_id.as_str());

    if changed {
        state.track = next_track.to_string();
        state::write(&state)?;
    }

    ui::section("Primer track");
    println!();
    if changed {
        ui::success(&format!("Switched track to {next_track}"));
    } else {
        ui::info(&format!("Track is already {}", ui::code(next_track)));
    }
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
            key: "Previous track".to_string(),
            value: previous_track,
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Active track".to_string(),
            value: state.track.clone(),
            value_color: Some(Color::Green),
        },
        ui::KeyValueRow {
            key: "Verified current milestone".to_string(),
            value: if current_milestone_verified {
                "yes".to_string()
            } else {
                "no".to_string()
            },
            value_color: Some(if current_milestone_verified {
                Color::Green
            } else {
                Color::Yellow
            }),
        },
        ui::KeyValueRow {
            key: "State file".to_string(),
            value: state.context_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);

    println!();
    ui::section("Next");
    let mut steps = vec![format!(
        "Run the {} to load {} guidance for {}",
        ui::reference("skill", "primer-build"),
        ui::code(&state.track),
        ui::code(&milestone.id)
    )];
    if current_milestone_verified {
        steps.push(format!(
            "Run the {} when you are ready to advance",
            ui::reference("skill", "primer-next-milestone")
        ));
    } else {
        steps.push(format!(
            "Switch back any time with {} or {}",
            ui::code("primer track learner"),
            ui::code("primer track builder")
        ));
    }
    ui::numbered_steps(&steps);

    Ok(())
}
