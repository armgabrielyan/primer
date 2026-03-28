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

fn run_primer(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_primer"))
        .args(args)
        .current_dir(cwd)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run primer")
}

fn write_recipe_files(recipe_dir: &Path, recipe_yaml: &str) {
    write_file(&recipe_dir.join("recipe.yaml"), recipe_yaml);
    write_file(&recipe_dir.join("README.md"), "# Demo recipe\n");
    write_file(
        &recipe_dir.join("milestones/01-start/spec.md"),
        "# Milestone 01\n",
    );
    write_file(
        &recipe_dir.join("milestones/01-start/explanation.md"),
        "# Explanation\n",
    );
    write_file(
        &recipe_dir.join("milestones/01-start/agent.md"),
        r#"# Agent Instructions

## Learner Track

Explain the milestone goal, then ask at least one question before coding?

## Builder Track

Implement the milestone directly and run verification before declaring it done.
"#,
    );
    write_file(
        &recipe_dir.join("milestones/01-start/demo.sh"),
        "#!/usr/bin/env bash\necho demo\n",
    );
    write_file(
        &recipe_dir.join("milestones/01-start/tests/check.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    );
}

#[test]
fn recipe_lint_passes_for_clean_recipe_directory() {
    let recipe_dir = temp_dir("recipe-lint-clean");
    write_recipe_files(
        &recipe_dir,
        r#"id: clean-demo
title: Clean Demo
description: Clean lint fixture.
difficulty: beginner
stack:
  id: python-cli
  label: Python CLI
  tools:
    - bash
tracks:
  learner:
    description: Learn in small steps.
  builder:
    description: Build directly.
milestones:
  - id: 01-start
    title: Start
    goal: Create the initial CLI entrypoint and persist tasks to a JSON file.
    verification_summary: Verify that a task can be added and listed back from persistent storage.
    expected_artifacts:
      - task_cli.py
      - tasks.json
    estimated_verify_minutes: 2
    split_if_stuck: Reduce scope to add plus list before touching richer command behavior.
    demo: Add a task and list it back from a JSON file.
    prerequisites:
      - bash
"#,
    );

    let output = run_primer(&recipe_dir, &["recipe", "lint"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Primer recipe lint"));
    assert!(stdout.contains("clean-demo"));
    assert!(stdout.contains("status: clean"));
    assert!(stdout.contains("Recipe lint passed."));
}

#[test]
fn recipe_lint_reports_quality_findings_from_repo_root() {
    let repo_root = temp_dir("recipe-lint-bad");
    let recipe_dir = repo_root.join("recipes").join("bad-demo");
    write_recipe_files(
        &recipe_dir,
        r#"id: bad-demo
title: Bad Demo
description: Bad lint fixture.
difficulty: beginner
stack:
  id: bash
  label: Bash
  tools:
    - bash
tracks:
  learner:
    description: Learn in small steps.
  builder:
    description: Build directly.
milestones:
  - id: 01-start
    title: Cleanup and Polish
    goal: Improve things.
    verification_summary: Looks good.
    expected_artifacts:
      - alpha.txt
      - beta.txt
      - gamma.txt
      - delta.txt
    demo: Change several things.
    prerequisites:
      - bash
"#,
    );

    let output = run_primer(&repo_root, &["recipe", "lint", "bad-demo"]);

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bad-demo"));
    assert!(stdout.contains("goal-vague"));
    assert!(stdout.contains("split-if-stuck-missing"));
    assert!(stdout.contains("verification-time-missing"));
    assert!(stdout.contains("multiple-capability-signals"));
}

#[test]
fn recipe_lint_json_prints_structured_output_on_failure() {
    let repo_root = temp_dir("recipe-lint-json");
    let recipe_dir = repo_root.join("recipes").join("bad-demo");
    write_recipe_files(
        &recipe_dir,
        r#"id: bad-demo
title: Bad Demo
description: Bad lint fixture.
difficulty: beginner
stack:
  id: bash
  label: Bash
  tools:
    - bash
tracks:
  learner:
    description: Learn in small steps.
  builder:
    description: Build directly.
milestones:
  - id: 01-start
    title: Cleanup and Polish
    goal: Improve things.
    verification_summary: Looks good.
    expected_artifacts:
      - alpha.txt
      - beta.txt
      - gamma.txt
      - delta.txt
    demo: Change several things.
    prerequisites:
      - bash
"#,
    );

    let output = run_primer(&repo_root, &["recipe", "lint", "bad-demo", "--json"]);

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON output");
    assert_eq!(json["checked"], 1);
    assert_eq!(json["recipes"][0]["recipe_id"], "bad-demo");
    assert!(
        json["warnings"]
            .as_u64()
            .expect("warnings should be a number")
            > 0
    );
    assert!(json["advice"].as_u64().expect("advice should be a number") > 0);
}

#[test]
fn recipe_lint_without_id_lints_all_local_recipes() {
    let repo_root = temp_dir("recipe-lint-all");
    let clean_recipe_dir = repo_root.join("recipes").join("clean-demo");
    let bad_recipe_dir = repo_root.join("recipes").join("bad-demo");

    write_recipe_files(
        &clean_recipe_dir,
        r#"id: clean-demo
title: Clean Demo
description: Clean lint fixture.
difficulty: beginner
stack:
  id: python-cli
  label: Python CLI
  tools:
    - bash
tracks:
  learner:
    description: Learn in small steps.
  builder:
    description: Build directly.
milestones:
  - id: 01-start
    title: Start
    goal: Create the initial CLI entrypoint and persist tasks to a JSON file.
    verification_summary: Verify that a task can be added and listed back from persistent storage.
    expected_artifacts:
      - task_cli.py
      - tasks.json
    estimated_verify_minutes: 2
    split_if_stuck: Reduce scope to add plus list before touching richer command behavior.
    demo: Add a task and list it back from a JSON file.
    prerequisites:
      - bash
"#,
    );
    write_recipe_files(
        &bad_recipe_dir,
        r#"id: bad-demo
title: Bad Demo
description: Bad lint fixture.
difficulty: beginner
stack:
  id: bash
  label: Bash
  tools:
    - bash
tracks:
  learner:
    description: Learn in small steps.
  builder:
    description: Build directly.
milestones:
  - id: 01-start
    title: Cleanup and Polish
    goal: Improve things.
    verification_summary: Looks good.
    expected_artifacts:
      - alpha.txt
      - beta.txt
      - gamma.txt
      - delta.txt
    demo: Change several things.
    prerequisites:
      - bash
"#,
    );

    let output = run_primer(&repo_root, &["recipe", "lint"]);

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clean-demo"));
    assert!(stdout.contains("bad-demo"));
    assert!(stdout.contains("2 recipe(s) checked"));
}
