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

fn run_primer(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_primer"))
        .args(args)
        .current_dir(cwd)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run primer")
}

fn verification_record_files(
    repo_root: &Path,
    workstream_id: &str,
    milestone_id: &str,
) -> Vec<PathBuf> {
    let dir = repo_root
        .join(".primer")
        .join("runtime")
        .join("workstreams")
        .join(workstream_id)
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

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("failed to read file")
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dirs");
    }
    fs::write(path, contents).expect("failed to write file");
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path).expect("missing file").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("failed to set permissions");
    }
}

#[cfg(windows)]
fn canonicalize_for_assert(path: &Path) -> PathBuf {
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
fn canonicalize_for_assert(path: &Path) -> PathBuf {
    fs::canonicalize(path).expect("failed to canonicalize path")
}

fn path_contains_suffix(path: &str, suffix: &str) -> bool {
    path.replace('\\', "/").contains(suffix)
}

fn write_verify_script(path: &Path, success_message: &str) {
    #[cfg(windows)]
    {
        let cmd_path = path.with_extension("cmd");
        write_file(
            &cmd_path,
            &format!("@echo off\r\necho {success_message}\r\n"),
        );
    }

    #[cfg(not(windows))]
    {
        write_file(
            path,
            &format!("#!/usr/bin/env bash\nset -euo pipefail\necho {success_message}\n"),
        );
        make_executable(path);
    }
}

#[test]
fn workstream_list_reports_when_no_workstreams_exist() {
    let repo = temp_dir("workstream-list-empty");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");

    let list = run_primer(&repo, &["workstream", "list"]);

    assert!(
        list.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("Primer workstreams"));
    assert!(stdout.contains("No repository-local Primer workstreams found"));
    assert!(stdout.contains("primer workstream analyze"));
    assert!(stdout.contains("primer workstream init <workstream-id> --goal ... --tool ..."));
}

#[test]
fn workstream_list_json_reports_when_no_workstreams_exist() {
    let repo = temp_dir("workstream-list-empty-json");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");
    let repo = canonicalize_for_assert(&repo);

    let list = run_primer(&repo, &["workstream", "list", "--json"]);

    assert!(
        list.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&list.stdout).expect("failed to parse JSON output");
    assert_eq!(json["repository"], repo.display().to_string());
    assert_eq!(json["active_workstream_id"], serde_json::Value::Null);
    assert_eq!(json["count"], 0);
    assert_eq!(
        json["workstreams"]
            .as_array()
            .expect("workstreams should be an array")
            .len(),
        0
    );
}

#[test]
fn workstream_analyze_suggests_repo_boundaries() {
    let repo = temp_dir("workstream-analyze");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");
    write_file(&repo.join("src/main.rs"), "fn main() {}\n");
    write_file(&repo.join("src/auth.rs"), "pub fn login() {}\n");
    write_file(&repo.join("tests/auth_test.rs"), "#[test]\nfn auth() {}\n");
    write_file(&repo.join("README.md"), "# Demo repo\n");

    let output = run_primer(&repo, &["workstream", "analyze"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Primer workstream analyze"));
    assert!(stdout.contains("Suggested first milestones"));
    assert!(stdout.contains("src"));
    assert!(stdout.contains("tests/auth_test.rs"));
    assert!(stdout.contains("primer workstream init <workstream-id> --goal ... --tool ..."));
}

#[test]
fn workstream_analyze_json_biases_toward_goal_matching_boundary() {
    let repo = temp_dir("workstream-analyze-json");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");
    write_file(
        &repo.join("packages/auth/src/lib.rs"),
        "pub fn token() {}\n",
    );
    write_file(
        &repo.join("packages/auth/tests/token_test.rs"),
        "#[test]\nfn token() {}\n",
    );
    write_file(
        &repo.join("packages/billing/src/lib.rs"),
        "pub fn bill() {}\n",
    );
    write_file(
        &repo.join("packages/billing/tests/billing_test.rs"),
        "#[test]\nfn billing() {}\n",
    );

    let output = run_primer(
        &repo,
        &[
            "workstream",
            "analyze",
            "--goal",
            "Harden auth tokens",
            "--json",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON output");
    let candidates = json["candidates"]
        .as_array()
        .expect("candidates should be an array");
    assert!(!candidates.is_empty());
    assert_eq!(candidates[0]["boundary"], "packages/auth");
    assert!(
        candidates[0]["goal_match_terms"]
            .as_array()
            .expect("goal_match_terms should be an array")
            .iter()
            .any(|value| value == "auth")
    );
}

#[test]
fn workstream_init_bootstraps_repo_local_workstream() {
    let repo = temp_dir("workstream-init");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");
    fs::write(repo.join("README.md"), "# Demo repo\n").expect("failed to write README.md");

    let output = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "auth-refactor",
            "--goal",
            "Reduce auth pipeline complexity",
            "--tool",
            "codex",
            "--track",
            "builder",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(repo.join("AGENTS.md").exists());
    assert!(repo.join(".agents/skills/primer-track/SKILL.md").exists());
    assert!(
        repo.join(".primer/workstreams/auth-refactor/workstream.yaml")
            .exists()
    );
    let intent_path = repo.join(".primer/workstreams/auth-refactor/workstream-intent.md");
    assert!(intent_path.exists());
    assert!(
        repo.join(
            ".primer/workstreams/auth-refactor/milestones/01-customize-first-milestone/spec.md"
        )
        .exists()
    );
    let context = fs::read_to_string(repo.join("AGENTS.md")).expect("failed to read AGENTS.md");
    assert!(context.contains("schema_version: 2"));
    assert!(context.contains("kind: workstream"));
    assert!(context.contains("id: auth-refactor"));
    assert!(context.contains("track: builder"));
    assert!(!context.contains("stack_id:"));
    let intent = fs::read_to_string(intent_path).expect("failed to read workstream intent");
    assert!(intent.contains("# Workstream Intent"));
    assert!(intent.contains("## Goal"));
    assert!(intent.contains("Reduce auth pipeline complexity"));
}

#[test]
fn workstream_flow_uses_repo_local_source_and_runtime_layout() {
    let repo = temp_dir("workstream-flow");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");
    fs::write(repo.join("src.txt"), "repo contents\n").expect("failed to write src.txt");

    let init = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "billing-webhooks",
            "--goal",
            "Harden webhook processing",
            "--tool",
            "codex",
        ],
    );
    assert!(
        init.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let status = run_primer(&repo, &["status"]);
    assert!(
        status.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("Workstream"));
    assert!(status_stdout.contains("billing-webhooks"));
    assert!(status_stdout.contains("01-customize-first-milestone"));

    let status_json = run_primer(&repo, &["status", "--json"]);
    assert!(
        status_json.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status_json.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&status_json.stdout).expect("failed to parse JSON output");
    assert_eq!(json["workflow_state"], "ready_to_build");
    assert_eq!(json["source"]["kind"], "workstream");
    assert_eq!(json["source"]["id"], "billing-webhooks");
    assert_eq!(
        json["current_milestone"]["id"],
        "01-customize-first-milestone"
    );
    assert_eq!(json["verified"], false);
    assert_eq!(json["verification"]["attempts"], 0);

    let build = run_primer(&repo, &["build"]);
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("Milestone contract"));
    assert!(build_stdout.contains("Workstream intent"));
    assert!(build_stdout.contains("Non-goals"));
    assert!(build_stdout.contains("Harden webhook processing"));

    let build_json = run_primer(&repo, &["build", "--json"]);
    assert!(
        build_json.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build_json.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&build_json.stdout).expect("failed to parse JSON output");
    assert_eq!(json["source"]["kind"], "workstream");
    assert_eq!(json["source"]["id"], "billing-webhooks");
    assert_eq!(
        json["current_milestone"]["id"],
        "01-customize-first-milestone"
    );
    assert_eq!(json["track"], "learner");
    assert!(
        json["intent_file"]
            .as_str()
            .expect("intent_file should be present")
            .contains("workstream-intent.md")
    );
    assert!(
        json["contract_markdown"]
            .as_str()
            .expect("contract_markdown should be present")
            .contains("Turn this scaffold into the first real brownfield milestone")
    );
    assert!(
        json["intent_markdown"]
            .as_str()
            .expect("intent_markdown should be present")
            .contains("## Goal")
    );
    assert!(
        json["intent_markdown"]
            .as_str()
            .expect("intent_markdown should be present")
            .contains("Harden webhook processing")
    );
    assert!(
        json["track_guidance_markdown"]
            .as_str()
            .expect("track_guidance_markdown should be present")
            .contains("Explain the repository area this workstream touches")
    );

    let verify = run_primer(&repo, &["verify"]);
    assert!(!verify.status.success());
    let verify_stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(path_contains_suffix(
        &verify_stdout,
        ".primer/workstreams/billing-webhooks"
    ));
    let verify_stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(verify_stderr.contains("real verification script"));
    let records =
        verification_record_files(&repo, "billing-webhooks", "01-customize-first-milestone");
    assert_eq!(records.len(), 1);

    let verify_json = run_primer(&repo, &["verify", "--json"]);
    assert!(!verify_json.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&verify_json.stdout).expect("failed to parse JSON output");
    assert_eq!(json["source"]["kind"], "workstream");
    assert_eq!(json["source"]["id"], "billing-webhooks");
    assert_eq!(json["milestone"]["id"], "01-customize-first-milestone");
    assert_eq!(json["outcome"], "failed");
    assert!(path_contains_suffix(
        json["command"]["script_path"]
            .as_str()
            .expect("script_path should be present"),
        ".primer/workstreams/billing-webhooks",
    ));
    assert!(
        json["command_stderr"]
            .as_str()
            .expect("command_stderr should be present")
            .contains("real verification script")
    );
}

#[test]
fn workstream_list_shows_initialized_workstreams_and_marks_the_active_one() {
    let repo = temp_dir("workstream-list");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");

    let first = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "auth-refactor",
            "--goal",
            "Reduce auth pipeline complexity",
            "--tool",
            "codex",
            "--track",
            "builder",
        ],
    );
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "billing-webhooks",
            "--goal",
            "Harden webhook processing",
            "--tool",
            "codex",
            "--track",
            "learner",
        ],
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let switch = run_primer(&repo, &["workstream", "switch", "auth-refactor"]);
    assert!(
        switch.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&switch.stderr)
    );

    let list = run_primer(&repo, &["workstream", "list"]);
    assert!(
        list.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("Active workstream"));
    assert!(stdout.contains("auth-refactor"));
    assert!(stdout.contains("billing-webhooks"));
    assert!(stdout.contains("Current milestone"));
    assert!(stdout.contains("01-customize-first-milestone"));
    assert!(stdout.contains("Verified"));
    assert!(stdout.contains("active"));
    assert!(stdout.contains("available"));
    assert!(stdout.contains(
        "Use primer workstream switch <workstream-id> to activate a different workstream"
    ));
}

#[test]
fn workstream_switch_restores_saved_milestone_and_verified_state() {
    let repo = temp_dir("workstream-resume");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");

    let auth_init = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "auth-refactor",
            "--goal",
            "Reduce auth pipeline complexity",
            "--tool",
            "codex",
            "--track",
            "builder",
        ],
    );
    assert!(
        auth_init.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&auth_init.stderr)
    );

    write_file(
        &repo.join(".primer/workstreams/auth-refactor/workstream.yaml"),
        r#"id: auth-refactor
title: Auth Refactor
goal: Reduce auth pipeline complexity
repository_root: /tmp/unused-for-test
tracks:
  learner:
    description: Learn safely.
  builder:
    description: Build directly.
milestones:
  - id: 01-customize-first-milestone
    title: Customize the first repo-specific milestone
    goal: Make the first auth milestone pass.
    verification_summary: Verification passes when the first auth verification script exits successfully.
  - id: 02-auth-observability
    title: Add auth observability
    goal: Add observability to the auth flow.
    verification_summary: Verification passes when the second auth verification script exits successfully.
"#,
    );
    write_verify_script(
        &repo.join(
            ".primer/workstreams/auth-refactor/milestones/01-customize-first-milestone/tests/verify.sh",
        ),
        "first auth milestone verified",
    );
    write_file(
        &repo.join(".primer/workstreams/auth-refactor/milestones/02-auth-observability/spec.md"),
        "# Milestone 02: Auth observability\n\nResume me after switching.\n",
    );
    write_file(
        &repo.join(".primer/workstreams/auth-refactor/milestones/02-auth-observability/agent.md"),
        r#"# Agent Instructions: 02-auth-observability

## Learner Track

Explain the observability work before coding and ask one question?

## Builder Track

Implement the observability milestone directly and run verification.
"#,
    );
    write_file(
        &repo.join(
            ".primer/workstreams/auth-refactor/milestones/02-auth-observability/explanation.md",
        ),
        "# Explanation: 02-auth-observability\n\nObserve auth state.\n",
    );
    write_verify_script(
        &repo.join(
            ".primer/workstreams/auth-refactor/milestones/02-auth-observability/tests/verify.sh",
        ),
        "second auth milestone verified",
    );

    let verify_first = run_primer(&repo, &["verify"]);
    assert!(
        verify_first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify_first.stderr)
    );
    let advance = run_primer(&repo, &["next-milestone"]);
    assert!(
        advance.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&advance.stderr)
    );
    let verify_second = run_primer(&repo, &["verify"]);
    assert!(
        verify_second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify_second.stderr)
    );

    let billing_init = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "billing-webhooks",
            "--goal",
            "Harden webhook processing",
            "--tool",
            "codex",
            "--track",
            "learner",
        ],
    );
    assert!(
        billing_init.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&billing_init.stderr)
    );

    let switch_back = run_primer(&repo, &["workstream", "switch", "auth-refactor"]);
    assert!(
        switch_back.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&switch_back.stderr)
    );
    let switch_stdout = String::from_utf8_lossy(&switch_back.stdout);
    assert!(switch_stdout.contains("restored saved progress"));
    assert!(switch_stdout.contains("02-auth-observability"));
    assert!(switch_stdout.contains("Verified current milestone"));
    assert!(switch_stdout.contains("yes"));

    let context = read(&repo.join("AGENTS.md"));
    assert!(context.contains("id: auth-refactor"));
    assert!(context.contains("milestone_id: 02-auth-observability"));
    assert!(context.contains("verified_milestone_id: 02-auth-observability"));
    assert!(context.contains("track: learner"));

    let status = run_primer(&repo, &["status"]);
    assert!(
        status.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("02-auth-observability"));
    assert!(status_stdout.contains("complete"));

    let list = run_primer(&repo, &["workstream", "list"]);
    assert!(
        list.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("Current milestone"));
    assert!(list_stdout.contains("02-auth-observability"));
    assert!(list_stdout.contains("Verified"));
    assert!(list_stdout.contains("yes"));

    let list_json = run_primer(&repo, &["workstream", "list", "--json"]);
    assert!(
        list_json.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&list_json.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&list_json.stdout).expect("failed to parse JSON output");
    assert_eq!(json["active_workstream_id"], "auth-refactor");
    assert_eq!(json["count"], 2);
    let workstreams = json["workstreams"]
        .as_array()
        .expect("workstreams should be an array");
    let auth_refactor = workstreams
        .iter()
        .find(|item| item["id"] == "auth-refactor")
        .expect("auth-refactor workstream missing");
    assert_eq!(auth_refactor["active"], true);
    assert_eq!(
        auth_refactor["current_milestone_id"],
        "02-auth-observability"
    );
    assert_eq!(
        auth_refactor["current_milestone_title"],
        "Add auth observability"
    );
    assert_eq!(auth_refactor["verified"], true);
    assert_eq!(auth_refactor["milestone_count"], 2);
    let billing_webhooks = workstreams
        .iter()
        .find(|item| item["id"] == "billing-webhooks")
        .expect("billing-webhooks workstream missing");
    assert_eq!(billing_webhooks["active"], false);
    assert_eq!(
        billing_webhooks["current_milestone_id"],
        "01-customize-first-milestone"
    );
    assert_eq!(billing_webhooks["verified"], false);
}

#[test]
fn workstream_init_can_activate_a_second_workstream_with_the_same_tool() {
    let repo = temp_dir("workstream-multi-init");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");

    let first = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "auth-refactor",
            "--goal",
            "Reduce auth pipeline complexity",
            "--tool",
            "codex",
            "--track",
            "builder",
        ],
    );
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "billing-webhooks",
            "--goal",
            "Harden webhook processing",
            "--tool",
            "codex",
            "--track",
            "learner",
        ],
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    assert!(
        repo.join(".primer/workstreams/auth-refactor/workstream.yaml")
            .exists()
    );
    assert!(
        repo.join(".primer/workstreams/billing-webhooks/workstream.yaml")
            .exists()
    );

    let context = read(&repo.join("AGENTS.md"));
    assert!(context.contains("kind: workstream"));
    assert!(context.contains("id: billing-webhooks"));
    assert!(context.contains("track: learner"));

    let status = run_primer(&repo, &["status"]);
    assert!(
        status.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status_stdout.contains("billing-webhooks"));
}

#[test]
fn workstream_switch_activates_existing_workstream_and_preserves_track() {
    let repo = temp_dir("workstream-switch");
    fs::create_dir_all(repo.join(".git")).expect("failed to create .git dir");

    let first = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "auth-refactor",
            "--goal",
            "Reduce auth pipeline complexity",
            "--tool",
            "codex",
            "--track",
            "builder",
        ],
    );
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    fs::write(
        repo.join(
            ".primer/workstreams/auth-refactor/milestones/01-customize-first-milestone/spec.md",
        ),
        "# Auth milestone\n\nSwitch back here later.\n",
    )
    .expect("failed to write auth workstream spec");

    let second = run_primer(
        &repo,
        &[
            "workstream",
            "init",
            "billing-webhooks",
            "--goal",
            "Harden webhook processing",
            "--tool",
            "codex",
            "--track",
            "learner",
        ],
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let switch = run_primer(&repo, &["workstream", "switch", "auth-refactor"]);
    assert!(
        switch.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&switch.stderr)
    );
    let switch_stdout = String::from_utf8_lossy(&switch.stdout);
    assert!(switch_stdout.contains("Previous workstream"));
    assert!(switch_stdout.contains("billing-webhooks"));
    assert!(switch_stdout.contains("auth-refactor"));

    let context = read(&repo.join("AGENTS.md"));
    assert!(context.contains("id: auth-refactor"));
    assert!(context.contains("track: learner"));
    assert!(context.contains("verified_milestone_id: null"));

    let build = run_primer(&repo, &["build"]);
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("Auth milestone"));
    assert!(build_stdout.contains("learner"));
}
