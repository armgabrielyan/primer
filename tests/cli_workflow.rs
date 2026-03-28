use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be monotonic enough for tests")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("primer-{label}-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(path, contents).expect("failed to write file");
}

fn make_executable(_path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(_path).expect("missing file").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(_path, perms).expect("failed to set permissions");
    }
}

#[cfg(windows)]
fn write_windows_check(path: &Path, body: &str) {
    write_file(path, body);
}

#[cfg(not(windows))]
fn write_windows_check(_path: &Path, _body: &str) {}

#[cfg(windows)]
fn canonicalize_for_state(path: &Path) -> PathBuf {
    let rendered = fs::canonicalize(path)
        .expect("failed to canonicalize path")
        .display()
        .to_string();

    if let Some(stripped) = rendered.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{}", stripped));
    }

    if let Some(stripped) = rendered.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped);
    }

    PathBuf::from(rendered)
}

#[cfg(not(windows))]
fn canonicalize_for_state(path: &Path) -> PathBuf {
    fs::canonicalize(path).expect("failed to canonicalize path")
}

fn setup_fixture(label: &str, verified_milestone_id: Option<&str>) -> (PathBuf, PathBuf) {
    let root = temp_dir(label);
    let primer_root = root.join("primer-root");
    let workspace_root = root.join("workspace");
    let recipe_path = primer_root.join("recipes/demo");

    write_file(
        &recipe_path.join("recipe.yaml"),
        r#"id: demo
title: Demo Recipe
difficulty: beginner
stack:
  id: test-stack
milestones:
  - id: 01-alpha
    title: Alpha
    goal: Create the alpha marker and keep the change isolated to the current workspace.
    verification_summary: Verification passes when milestone.ok exists in the workspace root.
    expected_artifacts:
      - milestone.ok
    estimated_verify_minutes: 1
    split_if_stuck: First confirm the file path and then re-run verification.
    prerequisites:
      - bash
  - id: 02-beta
    title: Beta
    goal: Create the beta marker in the workspace.
    verification_summary: Verification passes when the beta milestone script exits successfully.
    estimated_verify_minutes: 1
    prerequisites:
      - bash
"#,
    );

    write_file(
        &recipe_path.join("milestones/01-alpha/spec.md"),
        r#"# Milestone 01: Alpha

## Goal

Create the alpha marker in the workspace.
"#,
    );
    write_file(
        &recipe_path.join("milestones/01-alpha/explanation.md"),
        r#"# Explanation: 01-alpha

This is the alpha explanation.
"#,
    );
    write_file(
        &recipe_path.join("milestones/01-alpha/agent.md"),
        r#"# Agent Instructions: 01-alpha

## Learner Track

Explain the alpha task before coding.

## Builder Track

Implement the alpha task directly.
"#,
    );
    write_file(
        &recipe_path.join("milestones/01-alpha/tests/check.sh"),
        r#"#!/usr/bin/env bash
set -euo pipefail

[ -f milestone.ok ] || {
  echo "FAIL: milestone.ok is missing" >&2
  exit 1
}

echo "alpha check passed"
"#,
    );
    make_executable(&recipe_path.join("milestones/01-alpha/tests/check.sh"));
    write_windows_check(
        &recipe_path.join("milestones/01-alpha/tests/check.cmd"),
        "@echo off\r\nif not exist milestone.ok (\r\n  echo FAIL: milestone.ok is missing 1>&2\r\n  exit /b 1\r\n)\r\necho alpha check passed\r\n",
    );

    write_file(
        &recipe_path.join("milestones/02-beta/spec.md"),
        r#"# Milestone 02: Beta

## Goal

Create the beta marker in the workspace.
"#,
    );
    write_file(
        &recipe_path.join("milestones/02-beta/explanation.md"),
        r#"# Explanation: 02-beta

This is the beta explanation.
"#,
    );
    write_file(
        &recipe_path.join("milestones/02-beta/agent.md"),
        r#"# Agent Instructions: 02-beta

## Learner Track

Explain the beta task before coding.

## Builder Track

Implement the beta task directly.
"#,
    );
    write_file(
        &recipe_path.join("milestones/02-beta/tests/check.sh"),
        r#"#!/usr/bin/env bash
set -euo pipefail
echo "beta check passed"
"#,
    );
    make_executable(&recipe_path.join("milestones/02-beta/tests/check.sh"));
    write_windows_check(
        &recipe_path.join("milestones/02-beta/tests/check.cmd"),
        "@echo off\r\necho beta check passed\r\n",
    );

    fs::create_dir_all(&workspace_root).expect("failed to create workspace");
    let primer_root = canonicalize_for_state(&primer_root);
    let workspace_root = canonicalize_for_state(&workspace_root);
    let recipe_path = canonicalize_for_state(&primer_root.join("recipes/demo"));
    let verified = verified_milestone_id.unwrap_or("null");
    write_file(
        &workspace_root.join("CLAUDE.md"),
        &format!(
            "# Demo Context\n\n```yaml\nprimer_state:\n  recipe_id: demo\n  recipe_path: {}\n  workspace_root: {}\n  milestone_id: 01-alpha\n  verified_milestone_id: {}\n  track: learner\n  stack_id: test-stack\n```\n",
            recipe_path.display(),
            workspace_root.display(),
            verified
        ),
    );

    (primer_root, workspace_root)
}

fn run_primer(workspace_root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_primer"))
        .args(args)
        .current_dir(workspace_root)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run primer")
}

fn read_context(workspace_root: &Path) -> String {
    fs::read_to_string(workspace_root.join("CLAUDE.md")).expect("failed to read context")
}

fn verification_record_files(workspace_root: &Path, milestone_id: &str) -> Vec<PathBuf> {
    let dir = workspace_root
        .join(".primer")
        .join("runtime")
        .join("verifications")
        .join(milestone_id);
    let mut files = fs::read_dir(&dir)
        .expect("failed to read verification record dir")
        .map(|entry| {
            entry
                .expect("failed to read verification record entry")
                .path()
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn read_verification_record(path: &Path) -> serde_json::Value {
    let raw = fs::read_to_string(path).expect("failed to read verification record");
    serde_json::from_str(&raw).expect("failed to parse verification record")
}

#[test]
fn verify_updates_verified_milestone_on_success() {
    let (_primer_root, workspace_root) = setup_fixture("verify-success", None);
    write_file(&workspace_root.join("milestone.ok"), "ok\n");

    let output = run_primer(&workspace_root, &["verify"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let context = read_context(&workspace_root);
    assert!(context.contains("verified_milestone_id: 01-alpha"));

    let records = verification_record_files(&workspace_root, "01-alpha");
    assert_eq!(records.len(), 1);
    let record = read_verification_record(&records[0]);
    assert_eq!(record["outcome"], "passed");
    assert_eq!(record["verified_state_after"], true);
    assert_eq!(record["cleared_prior_verified_state"], false);
    assert_eq!(record["milestone_id"], "01-alpha");
}

#[test]
fn check_alias_still_verifies() {
    let (_primer_root, workspace_root) = setup_fixture("verify-alias", None);
    write_file(&workspace_root.join("milestone.ok"), "ok\n");

    let output = run_primer(&workspace_root, &["check"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let context = read_context(&workspace_root);
    assert!(context.contains("verified_milestone_id: 01-alpha"));
}

#[test]
fn verify_failure_keeps_unverified_state_unchanged() {
    let (_primer_root, workspace_root) = setup_fixture("verify-failure", None);

    let output = run_primer(&workspace_root, &["verify"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("milestone.ok is missing"));
    assert!(stderr.contains("verification failed"));
    let context = read_context(&workspace_root);
    assert!(context.contains("verified_milestone_id: null"));

    let records = verification_record_files(&workspace_root, "01-alpha");
    assert_eq!(records.len(), 1);
    let record = read_verification_record(&records[0]);
    assert_eq!(record["outcome"], "failed");
    assert_eq!(record["verified_state_after"], false);
    assert_eq!(record["cleared_prior_verified_state"], false);
}

#[test]
fn verify_failure_clears_prior_verified_state() {
    let (_primer_root, workspace_root) = setup_fixture("verify-failure-clears", Some("01-alpha"));

    let output = run_primer(&workspace_root, &["verify"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("milestone.ok is missing"));
    assert!(stderr.contains("current verified state was cleared"));
    let context = read_context(&workspace_root);
    assert!(context.contains("verified_milestone_id: null"));

    let records = verification_record_files(&workspace_root, "01-alpha");
    assert_eq!(records.len(), 1);
    let record = read_verification_record(&records[0]);
    assert_eq!(record["outcome"], "failed");
    assert_eq!(record["verified_state_after"], false);
    assert_eq!(record["cleared_prior_verified_state"], true);
}

#[test]
fn repeated_verifications_create_multiple_immutable_records() {
    let (_primer_root, workspace_root) = setup_fixture("verify-history", None);
    write_file(&workspace_root.join("milestone.ok"), "ok\n");

    let first = run_primer(&workspace_root, &["verify"]);
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    fs::remove_file(workspace_root.join("milestone.ok")).expect("failed to remove milestone.ok");

    let second = run_primer(&workspace_root, &["verify"]);
    assert!(!second.status.success());

    let records = verification_record_files(&workspace_root, "01-alpha");
    assert_eq!(records.len(), 2);

    let outcomes = records
        .iter()
        .map(|path| {
            read_verification_record(path)["outcome"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();
    assert!(outcomes.contains(&"passed".to_string()));
    assert!(outcomes.contains(&"failed".to_string()));
}

#[test]
fn next_milestone_requires_prior_verification() {
    let (_primer_root, workspace_root) = setup_fixture("next-guard", None);

    let output = run_primer(&workspace_root, &["next-milestone"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("run the skill primer-verify first"));
    let context = read_context(&workspace_root);
    assert!(context.contains("milestone_id: 01-alpha"));
    assert!(context.contains("verified_milestone_id: null"));
}

#[test]
fn next_milestone_advances_and_clears_verification() {
    let (_primer_root, workspace_root) = setup_fixture("next-success", Some("01-alpha"));

    let output = run_primer(&workspace_root, &["next-milestone"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let context = read_context(&workspace_root);
    assert!(context.contains("milestone_id: 02-beta"));
    assert!(context.contains("verified_milestone_id: null"));
}

#[test]
fn status_shows_ready_to_build_without_verification_history() {
    let (_primer_root, workspace_root) = setup_fixture("status-build", None);

    let output = run_primer(&workspace_root, &["status"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ready to build"));
    assert!(stdout.contains("Create the alpha marker and keep the change isolated"));
    assert!(stdout.contains("Verification passes when milestone.ok exists"));
    assert!(stdout.contains("Expected artifacts"));
    assert!(stdout.contains("First confirm the file path"));
    assert!(stdout.contains("1 minute"));
    assert!(stdout.contains("no verification attempts yet"));
    assert!(stdout.contains("blocked - milestone has not passed verification yet"));
    assert!(stdout.contains("Run the skill primer-build to work on"));
}

#[test]
fn status_shows_ready_to_advance_after_passing_verification() {
    let (_primer_root, workspace_root) = setup_fixture("status-advance", None);
    write_file(&workspace_root.join("milestone.ok"), "ok\n");

    let verify = run_primer(&workspace_root, &["verify"]);
    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let output = run_primer(&workspace_root, &["status"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ready to advance"));
    assert!(stdout.contains("passed in"));
    assert!(stdout.contains("open - current milestone is verified"));
    assert!(stdout.contains("Run the skill primer-next-milestone to advance"));
}

#[test]
fn status_shows_failed_latest_verification_and_retry_guidance() {
    let (_primer_root, workspace_root) = setup_fixture("status-retry", None);
    write_file(&workspace_root.join("milestone.ok"), "ok\n");

    let verify = run_primer(&workspace_root, &["verify"]);
    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    fs::remove_file(workspace_root.join("milestone.ok")).expect("failed to remove milestone.ok");

    let failed_verify = run_primer(&workspace_root, &["verify"]);
    assert!(!failed_verify.status.success());

    let output = run_primer(&workspace_root, &["status"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ready to verify"));
    assert!(stdout.contains("Failed attempts"));
    assert!(stdout.contains("Failure streak"));
    assert!(stdout.contains("retrying after 1 failed verification"));
    assert!(stdout.contains("failed in"));
    assert!(stdout.contains("blocked - latest verification failed"));
    assert!(stdout.contains("Run the skill primer-verify again"));
    assert!(stdout.contains("primer-explain for more context"));
}

#[test]
fn status_surfaces_if_stuck_guidance_after_two_failures() {
    let (_primer_root, workspace_root) = setup_fixture("status-stuck", None);

    let first = run_primer(&workspace_root, &["verify"]);
    assert!(!first.status.success());

    let second = run_primer(&workspace_root, &["verify"]);
    assert!(!second.status.success());

    let output = run_primer(&workspace_root, &["status"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("likely stuck after 2 consecutive failed verifications"));
    assert!(stdout.contains("Follow the milestone's If stuck guidance"));
    assert!(stdout.contains("First confirm the file path"));
}

#[test]
fn verify_failure_flags_scope_risk_after_three_failures() {
    let (_primer_root, workspace_root) = setup_fixture("verify-escalate", None);

    for _ in 0..2 {
        let output = run_primer(&workspace_root, &["verify"]);
        assert!(!output.status.success());
    }

    let output = run_primer(&workspace_root, &["verify"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "Verification history for 01-alpha: 3 attempts, 3 failed, current failure streak 3."
    ));
    assert!(stderr.contains("Run primer explain for more context before the next retry."));
    assert!(stderr.contains("If stuck: First confirm the file path and then re-run verification."));
    assert!(stderr.contains("This milestone may be too large or unclear. Consider splitting or clarifying it before more retries."));
}

#[test]
fn build_shows_current_spec_and_track_guidance() {
    let (_primer_root, workspace_root) = setup_fixture("build", None);

    let output = run_primer(&workspace_root, &["build"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Milestone 01: Alpha"));
    assert!(stdout.contains("Explain the alpha task before coding."));
    assert!(stdout.contains("Run the skill primer-verify when the milestone is complete"));
}

#[test]
fn explain_shows_current_milestone_explanation() {
    let (_primer_root, workspace_root) = setup_fixture("explain", Some("01-alpha"));

    let output = run_primer(&workspace_root, &["explain"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Explanation: 01-alpha"));
    assert!(stdout.contains("This is the alpha explanation."));
}
