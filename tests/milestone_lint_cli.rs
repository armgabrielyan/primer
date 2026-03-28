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

fn write_recipe_milestone(recipe_dir: &Path, milestone_id: &str, agent_body: &str) {
    write_file(
        &recipe_dir
            .join("milestones")
            .join(milestone_id)
            .join("spec.md"),
        "# Milestone\n",
    );
    write_file(
        &recipe_dir
            .join("milestones")
            .join(milestone_id)
            .join("explanation.md"),
        "# Explanation\n",
    );
    write_file(
        &recipe_dir
            .join("milestones")
            .join(milestone_id)
            .join("agent.md"),
        agent_body,
    );
    write_file(
        &recipe_dir
            .join("milestones")
            .join(milestone_id)
            .join("demo.sh"),
        "#!/usr/bin/env bash\necho demo\n",
    );
    write_file(
        &recipe_dir
            .join("milestones")
            .join(milestone_id)
            .join("tests/check.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    );
}

#[test]
fn milestone_lint_uses_active_workspace_current_milestone_by_default() {
    let root = temp_dir("milestone-lint-workspace");
    let recipe_dir = root.join("recipe");
    let workspace_dir = root.join("workspace");

    write_file(
        &recipe_dir.join("recipe.yaml"),
        r#"id: demo
title: Demo Recipe
difficulty: beginner
stack:
  id: test-stack
milestones:
  - id: 01-alpha
    title: Alpha
    goal: Create the alpha marker in the workspace and keep the change isolated.
    verification_summary: Verify that alpha output appears and the verification script exits successfully.
    expected_artifacts:
      - alpha.txt
    estimated_verify_minutes: 2
    split_if_stuck: Get one output path working before broadening the implementation.
    prerequisites:
      - bash
  - id: 02-beta
    title: Cleanup and Polish
    goal: Improve things.
    verification_summary: Looks good.
    expected_artifacts:
      - beta.txt
      - gamma.txt
      - delta.txt
      - epsilon.txt
    prerequisites:
      - bash
"#,
    );

    let valid_agent = r#"# Agent Instructions

## Learner Track

Explain the milestone and ask one question before coding?

## Builder Track

Implement the milestone directly and run verification before declaring it done.
"#;
    write_recipe_milestone(&recipe_dir, "01-alpha", valid_agent);
    write_recipe_milestone(&recipe_dir, "02-beta", valid_agent);

    let recipe_dir = fs::canonicalize(&recipe_dir).expect("failed to canonicalize recipe dir");
    let workspace_dir = fs::canonicalize(&{
        fs::create_dir_all(&workspace_dir).expect("failed to create workspace dir");
        workspace_dir
    })
    .expect("failed to canonicalize workspace dir");
    write_file(
        &workspace_dir.join("AGENTS.md"),
        &format!(
            "# Primer\n\n```yaml\nprimer_state:\n  schema_version: 2\n  source:\n    kind: recipe\n    id: demo\n    path: {}\n  workspace_root: {}\n  milestone_id: 02-beta\n  verified_milestone_id: null\n  track: learner\n  stack_id: test-stack\n```\n",
            recipe_dir.display(),
            workspace_dir.display()
        ),
    );

    let output = run_primer(&workspace_dir, &["milestone", "lint"]);

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("02-beta"));
    assert!(stdout.contains("goal-vague"));
}

#[test]
fn milestone_lint_infers_milestone_from_milestone_directory_path() {
    let recipe_dir = temp_dir("milestone-lint-path");
    write_file(
        &recipe_dir.join("recipe.yaml"),
        r#"id: clean-demo
title: Clean Demo
difficulty: beginner
stack:
  id: test-stack
milestones:
  - id: 01-alpha
    title: Alpha
    goal: Create the alpha marker in the workspace and keep the change isolated.
    verification_summary: Verify that alpha output appears and the verification script exits successfully.
    expected_artifacts:
      - alpha.txt
    estimated_verify_minutes: 2
    split_if_stuck: Get one output path working before broadening the implementation.
    prerequisites:
      - bash
"#,
    );
    write_recipe_milestone(
        &recipe_dir,
        "01-alpha",
        r#"# Agent Instructions

## Learner Track

Explain the milestone and ask one question before coding?

## Builder Track

Implement the milestone directly and run verification before declaring it done.
"#,
    );

    let milestone_dir = recipe_dir.join("milestones/01-alpha");
    let output = run_primer(
        &recipe_dir,
        &[
            "milestone",
            "lint",
            "--path",
            milestone_dir.to_str().expect("path should be utf-8"),
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("01-alpha"));
    assert!(stdout.contains("Milestone lint passed."));
}

#[test]
fn milestone_lint_supports_clean_workstream_milestones_without_demo_file() {
    let workstream_dir = temp_dir("milestone-lint-workstream");
    write_file(
        &workstream_dir.join("workstream.yaml"),
        r#"id: auth-refactor
title: Auth Refactor
milestones:
  - id: 01-map-boundary
    title: Map Boundary
    goal: Identify the auth boundary and capture the first safe repository-local change.
    verification_summary: Verify that the boundary notes exist and the verification script exits successfully.
    expected_artifacts:
      - docs/auth-boundary.md
    estimated_verify_minutes: 2
    split_if_stuck: Start with one boundary note before widening the first milestone.
"#,
    );
    write_file(
        &workstream_dir.join("milestones/01-map-boundary/spec.md"),
        "# Milestone\n",
    );
    write_file(
        &workstream_dir.join("milestones/01-map-boundary/explanation.md"),
        "# Explanation\n",
    );
    write_file(
        &workstream_dir.join("milestones/01-map-boundary/agent.md"),
        r#"# Agent Instructions

## Learner Track

Explain the repository area and ask one question before coding?

## Builder Track

Implement the milestone directly and run verification before declaring it done.
"#,
    );
    write_file(
        &workstream_dir.join("milestones/01-map-boundary/tests/verify.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    );

    let output = run_primer(&workstream_dir, &["milestone", "lint", "--json"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON output");
    assert_eq!(json["source"]["kind"], "workstream");
    assert_eq!(json["milestone"]["id"], "01-map-boundary");
    assert_eq!(json["errors"], 0);
    assert_eq!(json["warnings"], 0);
}
