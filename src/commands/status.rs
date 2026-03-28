use anyhow::{Context, Result};
use comfy_table::Color;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::cli::StatusArgs;
use crate::recipe::Milestone;
use crate::retry_guidance::{self, RetryAssessment, RetryLevel};
use crate::state;
use crate::ui;
use crate::verification_history::{self, VerificationOutcomeSummary};
use crate::workflow::{self, WorkflowSourceKind};

struct StatusData {
    workflow_state: WorkflowState,
    source_kind: WorkflowSourceKind,
    source_id: String,
    track: String,
    workspace_root: PathBuf,
    stack_id: Option<String>,
    current_milestone: Milestone,
    verified: bool,
    verification_summary: verification_history::VerificationSummary,
    retry_assessment: RetryAssessment,
    progress_current: usize,
    progress_total: usize,
    next_milestone: Option<Milestone>,
    state_file: PathBuf,
}

#[derive(Serialize)]
struct StatusJson {
    workflow_state: String,
    source: StatusSourceJson,
    track: String,
    workspace: String,
    stack_id: Option<String>,
    current_milestone: StatusMilestoneJson,
    verified: bool,
    verification: StatusVerificationJson,
    retry_signal: StatusRetrySignalJson,
    verification_gate: StatusVerificationGateJson,
    progress: StatusProgressJson,
    next_milestone: Option<StatusMilestoneRefJson>,
    state_file: String,
    next_steps: Vec<String>,
}

#[derive(Serialize)]
struct StatusSourceJson {
    kind: String,
    id: String,
}

#[derive(Serialize)]
struct StatusMilestoneJson {
    id: String,
    title: String,
    goal: Option<String>,
    verification_summary: Option<String>,
    estimated_verify_minutes: Option<u32>,
    expected_artifacts: Vec<String>,
    split_if_stuck: Option<String>,
}

#[derive(Serialize)]
struct StatusMilestoneRefJson {
    id: String,
    title: String,
}

#[derive(Serialize)]
struct StatusVerificationJson {
    attempts: usize,
    passed_attempts: usize,
    failed_attempts: usize,
    failure_streak: usize,
    last: Option<StatusVerificationAttemptJson>,
}

#[derive(Serialize)]
struct StatusVerificationAttemptJson {
    outcome: String,
    duration_ms: u128,
    formatted_duration: String,
    exit_code: Option<i32>,
    summary: String,
}

#[derive(Serialize)]
struct StatusRetrySignalJson {
    level: String,
    label: String,
}

#[derive(Serialize)]
struct StatusVerificationGateJson {
    state: String,
    summary: String,
}

#[derive(Serialize)]
struct StatusProgressJson {
    current: usize,
    total: usize,
}

pub fn run(workspace_hint: &Path, args: StatusArgs) -> Result<()> {
    let data = collect_status(workspace_hint)?;

    if args.json {
        let json = serde_json::to_string_pretty(&StatusJson::from_data(&data))
            .context("failed to serialize status output")?;
        println!("{json}");
        return Ok(());
    }

    render_human(&data);
    Ok(())
}

fn collect_status(workspace_hint: &Path) -> Result<StatusData> {
    let state = state::load_from_workspace(workspace_hint)?;
    let workflow = workflow::load(&state.source)?;
    let current = workflow::resolve_initial_milestone(&workflow, Some(&state.milestone_id))?;
    let current_index = workflow::milestone_index(&workflow, &state.milestone_id)?;
    let verified = state.verified_milestone_id.as_deref() == Some(state.milestone_id.as_str());
    let next = workflow.milestones.get(current_index + 1).cloned();
    let verification_summary = verification_history::summarize_for_milestone(&state)?;
    let retry_assessment = retry_guidance::assess(&verification_summary);
    let workflow_state = workflow_state(verified, verification_summary.attempts, next.is_none());

    Ok(StatusData {
        workflow_state,
        source_kind: state.source.kind,
        source_id: state.source.id,
        track: state.track,
        workspace_root: state.workspace_root,
        stack_id: state.stack_id,
        current_milestone: current.clone(),
        verified,
        verification_summary,
        retry_assessment,
        progress_current: current_index + 1,
        progress_total: workflow.milestones.len(),
        next_milestone: next,
        state_file: state.context_path,
    })
}

fn render_human(data: &StatusData) {
    ui::section("Primer status");
    println!();

    let mut rows = vec![
        ui::KeyValueRow {
            key: "Workflow state".to_string(),
            value: data.workflow_state.label().to_string(),
            value_color: Some(data.workflow_state.color()),
        },
        ui::KeyValueRow {
            key: data.source_kind.label().to_string(),
            value: data.source_id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: data.track.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workspace".to_string(),
            value: data.workspace_root.display().to_string(),
            value_color: Some(Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!(
                "{} ({})",
                data.current_milestone.id, data.current_milestone.title
            ),
            value_color: None,
        },
    ];

    if let Some(stack_id) = data.stack_id.as_ref() {
        rows.push(ui::KeyValueRow {
            key: "Stack".to_string(),
            value: stack_id.clone(),
            value_color: None,
        });
    }
    if let Some(goal) = data.current_milestone.goal.as_ref() {
        rows.push(ui::KeyValueRow {
            key: "Goal".to_string(),
            value: goal.clone(),
            value_color: None,
        });
    }
    if let Some(summary) = data.current_milestone.verification_summary.as_ref() {
        rows.push(ui::KeyValueRow {
            key: "Verification summary".to_string(),
            value: summary.clone(),
            value_color: None,
        });
    }
    if let Some(minutes) = data.current_milestone.estimated_verify_minutes {
        rows.push(ui::KeyValueRow {
            key: "Estimated verify time".to_string(),
            value: if minutes == 1 {
                "1 minute".to_string()
            } else {
                format!("{minutes} minutes")
            },
            value_color: None,
        });
    }
    if !data.current_milestone.expected_artifacts.is_empty() {
        rows.push(ui::KeyValueRow {
            key: "Expected artifacts".to_string(),
            value: data.current_milestone.expected_artifacts.join(", "),
            value_color: None,
        });
    }
    if let Some(split_if_stuck) = data.current_milestone.split_if_stuck.as_ref() {
        rows.push(ui::KeyValueRow {
            key: "If stuck".to_string(),
            value: split_if_stuck.clone(),
            value_color: None,
        });
    }

    rows.extend([
        ui::KeyValueRow {
            key: "Verified".to_string(),
            value: yes_no(data.verified).to_string(),
            value_color: Some(if data.verified {
                Color::Green
            } else {
                Color::Yellow
            }),
        },
        ui::KeyValueRow {
            key: "Verification attempts".to_string(),
            value: data.verification_summary.attempts.to_string(),
            value_color: None,
        },
    ]);

    if data.verification_summary.failed_attempts > 0 {
        rows.push(ui::KeyValueRow {
            key: "Failed attempts".to_string(),
            value: data.verification_summary.failed_attempts.to_string(),
            value_color: None,
        });
    }
    if data.retry_assessment.failure_streak > 0 {
        rows.push(ui::KeyValueRow {
            key: "Failure streak".to_string(),
            value: data.retry_assessment.failure_streak.to_string(),
            value_color: Some(Color::Red),
        });
        rows.push(ui::KeyValueRow {
            key: "Retry signal".to_string(),
            value: data.retry_assessment.label(),
            value_color: Some(retry_signal_color(data.retry_assessment.level)),
        });
    }

    rows.extend([
        ui::KeyValueRow {
            key: "Last verification".to_string(),
            value: last_verification_value(&data.verification_summary),
            value_color: Some(last_verification_color(&data.verification_summary)),
        },
        ui::KeyValueRow {
            key: "Verification gate".to_string(),
            value: verification_gate_value(data.verified, &data.verification_summary),
            value_color: Some(verification_gate_color(
                data.verified,
                &data.verification_summary,
            )),
        },
        ui::KeyValueRow {
            key: "Progress".to_string(),
            value: format!("{}/{}", data.progress_current, data.progress_total),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Next milestone".to_string(),
            value: data
                .next_milestone
                .as_ref()
                .map(|m| format!("{} ({})", m.id, m.title))
                .unwrap_or_else(|| "complete".to_string()),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "State file".to_string(),
            value: data.state_file.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);
    ui::key_value_table(&rows);

    println!();
    ui::section("Next");
    ui::numbered_steps(&human_next_steps(data));
}

impl StatusJson {
    fn from_data(data: &StatusData) -> Self {
        StatusJson {
            workflow_state: data.workflow_state.key().to_string(),
            source: StatusSourceJson {
                kind: data.source_kind.as_str().to_string(),
                id: data.source_id.clone(),
            },
            track: data.track.clone(),
            workspace: data.workspace_root.display().to_string(),
            stack_id: data.stack_id.clone(),
            current_milestone: StatusMilestoneJson {
                id: data.current_milestone.id.clone(),
                title: data.current_milestone.title.clone(),
                goal: data.current_milestone.goal.clone(),
                verification_summary: data.current_milestone.verification_summary.clone(),
                estimated_verify_minutes: data.current_milestone.estimated_verify_minutes,
                expected_artifacts: data.current_milestone.expected_artifacts.clone(),
                split_if_stuck: data.current_milestone.split_if_stuck.clone(),
            },
            verified: data.verified,
            verification: StatusVerificationJson {
                attempts: data.verification_summary.attempts,
                passed_attempts: data.verification_summary.passed_attempts,
                failed_attempts: data.verification_summary.failed_attempts,
                failure_streak: data.verification_summary.failure_streak,
                last: data.verification_summary.last.as_ref().map(|last| {
                    StatusVerificationAttemptJson {
                        outcome: verification_outcome_key(last.outcome).to_string(),
                        duration_ms: last.duration_ms,
                        formatted_duration: format_duration_ms(last.duration_ms),
                        exit_code: last.exit_code,
                        summary: last_verification_value(&data.verification_summary),
                    }
                }),
            },
            retry_signal: StatusRetrySignalJson {
                level: retry_level_key(data.retry_assessment.level).to_string(),
                label: data.retry_assessment.label(),
            },
            verification_gate: StatusVerificationGateJson {
                state: verification_gate_state_key(data.verified).to_string(),
                summary: verification_gate_value(data.verified, &data.verification_summary),
            },
            progress: StatusProgressJson {
                current: data.progress_current,
                total: data.progress_total,
            },
            next_milestone: data
                .next_milestone
                .as_ref()
                .map(|milestone| StatusMilestoneRefJson {
                    id: milestone.id.clone(),
                    title: milestone.title.clone(),
                }),
            state_file: data.state_file.display().to_string(),
            next_steps: json_next_steps(data),
        }
    }
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

    fn key(self) -> &'static str {
        match self {
            WorkflowState::ReadyToBuild => "ready_to_build",
            WorkflowState::ReadyToVerify => "ready_to_verify",
            WorkflowState::ReadyToAdvance => "ready_to_advance",
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

fn human_next_steps(data: &StatusData) -> Vec<String> {
    match data.workflow_state {
        WorkflowState::ReadyToAdvance => {
            let next = data
                .next_milestone
                .as_ref()
                .expect("next milestone should exist when ready to advance");
            vec![
                format!(
                    "Run the {} to advance to {}",
                    ui::reference("skill", "primer-next-milestone"),
                    ui::code(&next.id)
                ),
                format!(
                    "Read the next milestone contract and explanation for {}",
                    ui::code(&next.id)
                ),
            ]
        }
        WorkflowState::Complete => vec!["Workflow is complete.".to_string()],
        WorkflowState::ReadyToVerify => {
            let mut steps = vec![
                format!(
                    "Fix the current milestone in {}",
                    ui::code(data.workspace_root.display().to_string())
                ),
                format!(
                    "Run the {} again for {}",
                    ui::reference("skill", "primer-verify"),
                    ui::code(&data.current_milestone.id)
                ),
            ];
            if data.retry_assessment.should_suggest_explain() {
                steps.push(format!(
                    "If you are stuck, run the {} for more context",
                    ui::reference("skill", "primer-explain")
                ));
            }
            if data.retry_assessment.should_surface_if_stuck() {
                if let Some(split_if_stuck) = data.current_milestone.split_if_stuck.as_ref() {
                    steps.push(format!(
                        "Follow the milestone's If stuck guidance: {split_if_stuck}"
                    ));
                }
                steps.push(format!(
                    "If a different mode would help, switch tracks with {} or {}",
                    ui::code("primer track learner"),
                    ui::code("primer track builder")
                ));
            }
            if data.retry_assessment.should_flag_scope_risk() {
                steps.push(
                    "If failures keep repeating, this milestone may need to be split or clarified before more retries."
                        .to_string(),
                );
            }
            steps
        }
        WorkflowState::ReadyToBuild => vec![
            format!(
                "Run the {} to work on {}",
                ui::reference("skill", "primer-build"),
                ui::code(&data.current_milestone.id)
            ),
            format!(
                "Run the {} when the milestone is complete",
                ui::reference("skill", "primer-verify")
            ),
        ],
    }
}

fn json_next_steps(data: &StatusData) -> Vec<String> {
    match data.workflow_state {
        WorkflowState::ReadyToAdvance => {
            let next = data
                .next_milestone
                .as_ref()
                .expect("next milestone should exist when ready to advance");
            vec![
                format!("Run primer next-milestone to advance to {}", next.id),
                format!(
                    "Read the next milestone contract and explanation for {}",
                    next.id
                ),
            ]
        }
        WorkflowState::Complete => vec!["Workflow is complete.".to_string()],
        WorkflowState::ReadyToVerify => {
            let mut steps = vec![
                format!(
                    "Fix the current milestone in {}",
                    data.workspace_root.display()
                ),
                format!("Run primer verify again for {}", data.current_milestone.id),
            ];
            if data.retry_assessment.should_suggest_explain() {
                steps.push("If you are stuck, run primer explain for more context".to_string());
            }
            if data.retry_assessment.should_surface_if_stuck() {
                if let Some(split_if_stuck) = data.current_milestone.split_if_stuck.as_ref() {
                    steps.push(format!(
                        "Follow the milestone's If stuck guidance: {split_if_stuck}"
                    ));
                }
                steps.push(
                    "If a different mode would help, switch tracks with primer track learner or primer track builder"
                        .to_string(),
                );
            }
            if data.retry_assessment.should_flag_scope_risk() {
                steps.push(
                    "If failures keep repeating, this milestone may need to be split or clarified before more retries."
                        .to_string(),
                );
            }
            steps
        }
        WorkflowState::ReadyToBuild => vec![
            format!("Run primer build to work on {}", data.current_milestone.id),
            "Run primer verify when the milestone is complete".to_string(),
        ],
    }
}

fn last_verification_value(summary: &verification_history::VerificationSummary) -> String {
    let Some(last) = summary.last.as_ref() else {
        return "no verification attempts yet".to_string();
    };

    let outcome = verification_outcome_key(last.outcome);
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

fn verification_gate_state_key(verified: bool) -> &'static str {
    if verified { "open" } else { "blocked" }
}

fn verification_outcome_key(outcome: VerificationOutcomeSummary) -> &'static str {
    match outcome {
        VerificationOutcomeSummary::Passed => "passed",
        VerificationOutcomeSummary::Failed => "failed",
    }
}

fn retry_level_key(level: RetryLevel) -> &'static str {
    match level {
        RetryLevel::Clear => "clear",
        RetryLevel::Retrying => "retrying",
        RetryLevel::Stuck => "stuck",
        RetryLevel::Escalating => "escalating",
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

fn retry_signal_color(level: RetryLevel) -> Color {
    match level {
        RetryLevel::Clear => Color::Green,
        RetryLevel::Retrying => Color::Yellow,
        RetryLevel::Stuck | RetryLevel::Escalating => Color::Red,
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
