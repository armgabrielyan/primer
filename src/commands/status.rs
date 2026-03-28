use anyhow::Result;
use comfy_table::Color;
use std::path::Path;

use crate::recipe;
use crate::state;
use crate::ui;
use crate::verification_history::{self, VerificationOutcomeSummary};

pub fn run(workspace_hint: &Path) -> Result<()> {
    let state = state::load_from_workspace(workspace_hint)?;
    let recipe = recipe::load_from_path(&state.recipe_path)?;
    let current = recipe::resolve_initial_milestone(&recipe, Some(&state.milestone_id))?;
    let current_index = recipe::milestone_index(&recipe, &state.milestone_id)?;
    let verified = state.verified_milestone_id.as_deref() == Some(state.milestone_id.as_str());
    let next = recipe.milestones.get(current_index + 1);
    let verification_summary = verification_history::summarize_for_milestone(&state)?;
    let workflow_state = workflow_state(verified, verification_summary.attempts, next.is_none());

    ui::section("Primer status");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Workflow state".to_string(),
            value: workflow_state.label().to_string(),
            value_color: Some(workflow_state.color()),
        },
        ui::KeyValueRow {
            key: "Recipe".to_string(),
            value: state.recipe_id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: state.track.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Stack".to_string(),
            value: state.stack_id.clone(),
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
            value_color: Some(if verified {
                Color::Green
            } else {
                Color::Yellow
            }),
        },
        ui::KeyValueRow {
            key: "Verification attempts".to_string(),
            value: verification_summary.attempts.to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Last verification".to_string(),
            value: last_verification_value(&verification_summary),
            value_color: Some(last_verification_color(&verification_summary)),
        },
        ui::KeyValueRow {
            key: "Verification gate".to_string(),
            value: verification_gate_value(verified, &verification_summary),
            value_color: Some(verification_gate_color(verified, &verification_summary)),
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
    match workflow_state {
        WorkflowState::ReadyToAdvance => {
            let next = next.expect("next milestone should exist when ready to advance");
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
        }
        WorkflowState::Complete => {
            ui::numbered_steps(&["Recipe is complete.".to_string()]);
        }
        WorkflowState::ReadyToVerify => {
            let mut steps = vec![
                format!(
                    "Fix the current milestone in {}",
                    ui::code(state.workspace_root.display().to_string())
                ),
                format!(
                    "Run the {} again for {}",
                    ui::reference("skill", "primer-verify"),
                    ui::code(&current.id)
                ),
            ];
            if matches!(
                verification_summary.last.as_ref().map(|last| last.outcome),
                Some(VerificationOutcomeSummary::Failed)
            ) {
                steps.push(format!(
                    "If you are stuck, run the {} for more context",
                    ui::reference("skill", "primer-explain")
                ));
            }
            ui::numbered_steps(&steps);
        }
        WorkflowState::ReadyToBuild => ui::numbered_steps(&[
            format!(
                "Run the {} to work on {}",
                ui::reference("skill", "primer-build"),
                ui::code(&current.id)
            ),
            format!(
                "Run the {} when the milestone is complete",
                ui::reference("skill", "primer-verify")
            ),
        ]),
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum WorkflowState {
    ReadyToBuild,
    ReadyToVerify,
    ReadyToAdvance,
    Complete,
}

impl WorkflowState {
    fn label(self) -> &'static str {
        match self {
            WorkflowState::ReadyToBuild => "ready to build",
            WorkflowState::ReadyToVerify => "ready to verify",
            WorkflowState::ReadyToAdvance => "ready to advance",
            WorkflowState::Complete => "complete",
        }
    }

    fn color(self) -> Color {
        match self {
            WorkflowState::ReadyToBuild => Color::Yellow,
            WorkflowState::ReadyToVerify => Color::Yellow,
            WorkflowState::ReadyToAdvance => Color::Green,
            WorkflowState::Complete => Color::Green,
        }
    }
}

fn workflow_state(verified: bool, attempts: usize, is_final_milestone: bool) -> WorkflowState {
    if verified && is_final_milestone {
        WorkflowState::Complete
    } else if verified {
        WorkflowState::ReadyToAdvance
    } else if attempts > 0 {
        WorkflowState::ReadyToVerify
    } else {
        WorkflowState::ReadyToBuild
    }
}

fn last_verification_value(summary: &verification_history::VerificationSummary) -> String {
    let Some(last) = summary.last.as_ref() else {
        return "no verification attempts yet".to_string();
    };

    let outcome = match last.outcome {
        VerificationOutcomeSummary::Passed => "passed",
        VerificationOutcomeSummary::Failed => "failed",
    };
    match last.exit_code {
        Some(exit_code) => format!(
            "{} in {} (exit {})",
            outcome,
            format_duration_ms(last.duration_ms),
            exit_code
        ),
        None => format!("{} in {}", outcome, format_duration_ms(last.duration_ms)),
    }
}

fn last_verification_color(summary: &verification_history::VerificationSummary) -> Color {
    match summary.last.as_ref().map(|last| last.outcome) {
        Some(VerificationOutcomeSummary::Passed) => Color::Green,
        Some(VerificationOutcomeSummary::Failed) => Color::Red,
        None => Color::DarkGrey,
    }
}

fn verification_gate_value(
    verified: bool,
    summary: &verification_history::VerificationSummary,
) -> String {
    if verified {
        "open - current milestone is verified".to_string()
    } else if matches!(
        summary.last.as_ref().map(|last| last.outcome),
        Some(VerificationOutcomeSummary::Failed)
    ) {
        "blocked - latest verification failed".to_string()
    } else {
        "blocked - milestone has not passed verification yet".to_string()
    }
}

fn verification_gate_color(
    verified: bool,
    summary: &verification_history::VerificationSummary,
) -> Color {
    if verified {
        Color::Green
    } else if matches!(
        summary.last.as_ref().map(|last| last.outcome),
        Some(VerificationOutcomeSummary::Failed)
    ) {
        Color::Red
    } else {
        Color::Yellow
    }
}

fn format_duration_ms(duration_ms: u128) -> String {
    if duration_ms < 1_000 {
        format!("{duration_ms} ms")
    } else if duration_ms < 60_000 {
        format!("{}.{:03} s", duration_ms / 1_000, duration_ms % 1_000)
    } else {
        let minutes = duration_ms / 60_000;
        let seconds = (duration_ms % 60_000) / 1_000;
        format!("{minutes}m {seconds}s")
    }
}
