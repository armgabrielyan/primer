use anyhow::{Result, bail};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::cli::NextMilestoneArgs;
use crate::recipe::Milestone;
use crate::state;
use crate::ui;
use crate::workflow;
use crate::workstream_resume;

#[derive(Serialize)]
struct NextMilestoneJson {
    source: NextMilestoneSourceJson,
    track: String,
    workspace: String,
    state_file: String,
    previous_milestone: NextMilestoneRefJson,
    current_milestone: NextMilestoneRefJson,
    advanced: bool,
    verification_cleared: bool,
    status: String,
    summary: String,
    spec_path: Option<String>,
    explanation_path: Option<String>,
    next_steps: Vec<String>,
}

#[derive(Serialize)]
struct NextMilestoneSourceJson {
    kind: String,
    id: String,
}

#[derive(Serialize)]
struct NextMilestoneRefJson {
    id: String,
    title: String,
}

enum NextMilestoneResult {
    Advanced {
        source_kind: String,
        source_id: String,
        track: String,
        workspace: PathBuf,
        state_file: PathBuf,
        previous_milestone: Box<Milestone>,
        current_milestone: Box<Milestone>,
        spec_path: PathBuf,
        explanation_path: PathBuf,
    },
    Complete {
        source_kind: String,
        source_id: String,
        track: String,
        workspace: PathBuf,
        state_file: PathBuf,
        current_milestone: Box<Milestone>,
    },
    Blocked {
        source_kind: String,
        source_id: String,
        track: String,
        workspace: PathBuf,
        state_file: PathBuf,
        current_milestone: Box<Milestone>,
    },
}

pub fn run(workspace_hint: &Path, args: NextMilestoneArgs) -> Result<()> {
    let result = collect_next_milestone_result(workspace_hint)?;

    match &result {
        NextMilestoneResult::Advanced { .. } => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&NextMilestoneJson::from_result(&result))?
                );
                return Ok(());
            }

            render_human(&result);
            Ok(())
        }
        NextMilestoneResult::Complete { .. } => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&NextMilestoneJson::from_result(&result))?
                );
                return Ok(());
            }

            render_human(&result);
            Ok(())
        }
        NextMilestoneResult::Blocked {
            current_milestone, ..
        } => {
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&NextMilestoneJson::from_result(&result))?
                );
            }
            bail!(
                "milestone {} is not verified yet; run the {} first",
                current_milestone.id,
                ui::reference("skill", "primer-verify")
            )
        }
    }
}

fn collect_next_milestone_result(workspace_hint: &Path) -> Result<NextMilestoneResult> {
    let mut state = state::load_from_workspace(workspace_hint)?;
    let workflow = workflow::load(&state.source)?;
    let current = workflow::resolve_initial_milestone(&workflow, Some(&state.milestone_id))?;
    let current = current.clone();
    let current_index = workflow::milestone_index(&workflow, &state.milestone_id)?;

    if state.verified_milestone_id.as_deref() != Some(current.id.as_str()) {
        return Ok(NextMilestoneResult::Blocked {
            source_kind: state.source.kind.as_str().to_string(),
            source_id: state.source.id.clone(),
            track: state.track.clone(),
            workspace: state.workspace_root.clone(),
            state_file: state.context_path.clone(),
            current_milestone: Box::new(current),
        });
    }

    let Some(next) = workflow.milestones.get(current_index + 1) else {
        return Ok(NextMilestoneResult::Complete {
            source_kind: state.source.kind.as_str().to_string(),
            source_id: state.source.id.clone(),
            track: state.track.clone(),
            workspace: state.workspace_root.clone(),
            state_file: state.context_path.clone(),
            current_milestone: Box::new(current),
        });
    };
    let next = next.clone();

    state.milestone_id = next.id.clone();
    state.verified_milestone_id = None;
    state::write(&state)?;
    workstream_resume::sync_from_state(&state)?;

    let spec_path = workflow
        .path
        .join("milestones")
        .join(&next.id)
        .join("spec.md");
    let explanation_path = workflow
        .path
        .join("milestones")
        .join(&next.id)
        .join("explanation.md");

    Ok(NextMilestoneResult::Advanced {
        source_kind: state.source.kind.as_str().to_string(),
        source_id: state.source.id,
        track: state.track,
        workspace: state.workspace_root,
        state_file: state.context_path,
        previous_milestone: Box::new(current),
        current_milestone: Box::new(next),
        spec_path,
        explanation_path,
    })
}

fn render_human(result: &NextMilestoneResult) {
    match result {
        NextMilestoneResult::Advanced {
            previous_milestone,
            current_milestone,
            track,
            state_file,
            spec_path,
            explanation_path,
            ..
        } => {
            ui::section("Primer next milestone");
            println!();
            ui::success(&format!("Advanced to {}", current_milestone.id));
            println!();
            ui::key_value_table(&[
                ui::KeyValueRow {
                    key: "Previous milestone".to_string(),
                    value: format!("{} ({})", previous_milestone.id, previous_milestone.title),
                    value_color: None,
                },
                ui::KeyValueRow {
                    key: "Current milestone".to_string(),
                    value: format!("{} ({})", current_milestone.id, current_milestone.title),
                    value_color: None,
                },
                ui::KeyValueRow {
                    key: "Track".to_string(),
                    value: track.clone(),
                    value_color: None,
                },
                ui::KeyValueRow {
                    key: "State file".to_string(),
                    value: state_file.display().to_string(),
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
                    ui::code(&current_milestone.id)
                ),
            ]);
        }
        NextMilestoneResult::Complete {
            current_milestone, ..
        } => {
            ui::section("Primer next milestone");
            println!();
            ui::success("Workflow complete");
            println!(
                "The final milestone {} is already verified. No state changes were made.",
                ui::code(&current_milestone.id)
            );
        }
        NextMilestoneResult::Blocked { .. } => {}
    }
}

impl NextMilestoneJson {
    fn from_result(result: &NextMilestoneResult) -> Self {
        match result {
            NextMilestoneResult::Advanced {
                source_kind,
                source_id,
                track,
                workspace,
                state_file,
                previous_milestone,
                current_milestone,
                spec_path,
                explanation_path,
            } => NextMilestoneJson {
                source: NextMilestoneSourceJson {
                    kind: source_kind.clone(),
                    id: source_id.clone(),
                },
                track: track.clone(),
                workspace: workspace.display().to_string(),
                state_file: state_file.display().to_string(),
                previous_milestone: NextMilestoneRefJson::from_milestone(previous_milestone),
                current_milestone: NextMilestoneRefJson::from_milestone(current_milestone),
                advanced: true,
                verification_cleared: true,
                status: "advanced".to_string(),
                summary: format!(
                    "advanced from {} to {} and cleared verified state",
                    previous_milestone.id, current_milestone.id
                ),
                spec_path: Some(spec_path.display().to_string()),
                explanation_path: Some(explanation_path.display().to_string()),
                next_steps: vec![
                    format!("Read {}", spec_path.display()),
                    format!(
                        "Read {} for the deep-dive explanation",
                        explanation_path.display()
                    ),
                    format!("Run primer build to work on {}", current_milestone.id),
                ],
            },
            NextMilestoneResult::Complete {
                source_kind,
                source_id,
                track,
                workspace,
                state_file,
                current_milestone,
            } => NextMilestoneJson {
                source: NextMilestoneSourceJson {
                    kind: source_kind.clone(),
                    id: source_id.clone(),
                },
                track: track.clone(),
                workspace: workspace.display().to_string(),
                state_file: state_file.display().to_string(),
                previous_milestone: NextMilestoneRefJson::from_milestone(current_milestone),
                current_milestone: NextMilestoneRefJson::from_milestone(current_milestone),
                advanced: false,
                verification_cleared: false,
                status: "complete".to_string(),
                summary: format!(
                    "the final milestone {} is already verified; no state changes were made",
                    current_milestone.id
                ),
                spec_path: None,
                explanation_path: None,
                next_steps: vec!["Workflow is complete.".to_string()],
            },
            NextMilestoneResult::Blocked {
                source_kind,
                source_id,
                track,
                workspace,
                state_file,
                current_milestone,
            } => NextMilestoneJson {
                source: NextMilestoneSourceJson {
                    kind: source_kind.clone(),
                    id: source_id.clone(),
                },
                track: track.clone(),
                workspace: workspace.display().to_string(),
                state_file: state_file.display().to_string(),
                previous_milestone: NextMilestoneRefJson::from_milestone(current_milestone),
                current_milestone: NextMilestoneRefJson::from_milestone(current_milestone),
                advanced: false,
                verification_cleared: false,
                status: "blocked".to_string(),
                summary: format!(
                    "milestone {} is not verified yet; run primer verify first",
                    current_milestone.id
                ),
                spec_path: None,
                explanation_path: None,
                next_steps: vec![
                    format!(
                        "Run primer verify for {} before advancing",
                        current_milestone.id
                    ),
                    "Run primer status to inspect the current verification gate".to_string(),
                ],
            },
        }
    }
}

impl NextMilestoneRefJson {
    fn from_milestone(milestone: &Milestone) -> Self {
        NextMilestoneRefJson {
            id: milestone.id.clone(),
            title: milestone.title.clone(),
        }
    }
}
