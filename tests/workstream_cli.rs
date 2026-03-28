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

    let build = run_primer(&repo, &["build"]);
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("Turn this scaffold into the first real brownfield milestone"));

    let verify = run_primer(&repo, &["verify"]);
    assert!(!verify.status.success());
    let verify_stderr = String::from_utf8_lossy(&verify.stderr);
    assert!(verify_stderr.contains("TODO: replace .primer/workstreams/billing-webhooks"));
    let records =
        verification_record_files(&repo, "billing-webhooks", "01-customize-first-milestone");
    assert_eq!(records.len(), 1);
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
