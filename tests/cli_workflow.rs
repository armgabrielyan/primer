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
    prerequisites:
      - bash
  - id: 02-beta
    title: Beta
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
    let primer_root = fs::canonicalize(&primer_root).expect("failed to canonicalize primer root");
    let workspace_root =
        fs::canonicalize(&workspace_root).expect("failed to canonicalize workspace root");
    let recipe_path = fs::canonicalize(primer_root.join("recipes/demo"))
        .expect("failed to canonicalize recipe path");
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

#[test]
fn check_updates_verified_milestone_on_success() {
    let (_primer_root, workspace_root) = setup_fixture("check-success", None);
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
fn check_failure_keeps_state_unchanged() {
    let (_primer_root, workspace_root) = setup_fixture("check-failure", None);

    let output = run_primer(&workspace_root, &["check"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("state was not updated"));
    let context = read_context(&workspace_root);
    assert!(context.contains("verified_milestone_id: null"));
}

#[test]
fn next_milestone_requires_prior_verification() {
    let (_primer_root, workspace_root) = setup_fixture("next-guard", None);

    let output = run_primer(&workspace_root, &["next-milestone"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("run the skill primer-check first"));
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
    assert!(stdout.contains("Run the skill primer-check when the milestone is complete"));
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
