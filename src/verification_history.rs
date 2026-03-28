use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::state::PrimerState;

static RECORD_COUNTER: AtomicU64 = AtomicU64::new(1);

pub enum VerificationOutcome {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationOutcomeSummary {
    Passed,
    Failed,
}

pub struct VerificationCommand<'a> {
    pub program: &'a OsStr,
    pub args: &'a [OsString],
    pub script: &'a Path,
}

pub struct VerificationSummary {
    pub attempts: usize,
    pub passed_attempts: usize,
    pub failed_attempts: usize,
    pub failure_streak: usize,
    pub last: Option<VerificationRecordSummary>,
}

pub struct VerificationRecordSummary {
    pub outcome: VerificationOutcomeSummary,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
}

#[derive(Deserialize, Serialize)]
struct VerificationRecord {
    schema_version: u32,
    recipe_id: String,
    workspace_root: PathBuf,
    milestone_id: String,
    track: String,
    outcome: String,
    recorded_at_unix_ms: u128,
    duration_ms: u128,
    verified_state_after: bool,
    cleared_prior_verified_state: bool,
    exit_code: Option<i32>,
    summary: Option<String>,
    command: VerificationCommandRecord,
}

#[derive(Deserialize, Serialize)]
struct VerificationCommandRecord {
    program: String,
    args: Vec<String>,
    script_path: PathBuf,
}

pub fn summarize_for_milestone(state: &PrimerState) -> Result<VerificationSummary> {
    let records_dir = state
        .workspace_root
        .join(".primer")
        .join("runtime")
        .join("verifications")
        .join(&state.milestone_id);

    if !records_dir.is_dir() {
        return Ok(VerificationSummary {
            attempts: 0,
            passed_attempts: 0,
            failed_attempts: 0,
            failure_streak: 0,
            last: None,
        });
    }

    let mut loaded_records = Vec::new();

    for entry in fs::read_dir(&records_dir)
        .with_context(|| format!("failed to read {}", records_dir.display()))?
    {
        let entry = entry
            .with_context(|| format!("failed to read entry under {}", records_dir.display()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let record: VerificationRecord = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse verification record {}", path.display()))?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        loaded_records.push((
            record.recorded_at_unix_ms,
            filename,
            VerificationRecordSummary {
                outcome: match record.outcome.as_str() {
                    "passed" => VerificationOutcomeSummary::Passed,
                    _ => VerificationOutcomeSummary::Failed,
                },
                duration_ms: record.duration_ms,
                exit_code: record.exit_code,
            },
        ));
    }

    loaded_records.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let attempts = loaded_records.len();
    let mut passed_attempts = 0usize;
    let mut failed_attempts = 0usize;
    let mut failure_streak = 0usize;

    for (_, _, record) in &loaded_records {
        match record.outcome {
            VerificationOutcomeSummary::Passed => {
                passed_attempts += 1;
                failure_streak = 0;
            }
            VerificationOutcomeSummary::Failed => {
                failed_attempts += 1;
                failure_streak += 1;
            }
        }
    }

    Ok(VerificationSummary {
        attempts,
        passed_attempts,
        failed_attempts,
        failure_streak,
        last: loaded_records
            .into_iter()
            .last()
            .map(|(_, _, record)| record),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn write_record(
    state: &PrimerState,
    command: &VerificationCommand<'_>,
    outcome: VerificationOutcome,
    duration: Duration,
    exit_code: Option<i32>,
    verified_state_after: bool,
    cleared_prior_verified_state: bool,
    summary: Option<&str>,
) -> Result<PathBuf> {
    let records_dir = state
        .workspace_root
        .join(".primer")
        .join("runtime")
        .join("verifications")
        .join(&state.milestone_id);
    fs::create_dir_all(&records_dir)
        .with_context(|| format!("failed to create {}", records_dir.display()))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is set before UNIX_EPOCH")?;
    let counter = RECORD_COUNTER.fetch_add(1, Ordering::Relaxed);
    let filename = format!(
        "{}-{:09}-{}-{}.json",
        timestamp.as_secs(),
        timestamp.subsec_nanos(),
        std::process::id(),
        counter
    );
    let path = records_dir.join(filename);

    let record = VerificationRecord {
        schema_version: 1,
        recipe_id: state.recipe_id.clone(),
        workspace_root: state.workspace_root.clone(),
        milestone_id: state.milestone_id.clone(),
        track: state.track.clone(),
        outcome: match outcome {
            VerificationOutcome::Passed => "passed".to_string(),
            VerificationOutcome::Failed => "failed".to_string(),
        },
        recorded_at_unix_ms: timestamp.as_millis(),
        duration_ms: duration.as_millis(),
        verified_state_after,
        cleared_prior_verified_state,
        exit_code,
        summary: summary.map(ToOwned::to_owned),
        command: VerificationCommandRecord {
            program: command.program.to_string_lossy().into_owned(),
            args: command
                .args
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
            script_path: command.script.to_path_buf(),
        },
    };

    let json =
        serde_json::to_string_pretty(&record).context("failed to serialize verification record")?;
    fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}
