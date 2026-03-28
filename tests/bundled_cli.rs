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

#[test]
fn list_uses_bundled_recipes_outside_repo() {
    let cwd = temp_dir("bundled-list");
    let output = run_primer(&cwd, &["list"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("operating-system"));
    assert!(stdout.contains("bundled"));
}

#[test]
fn init_uses_bundled_recipe_without_external_repo() {
    let root = temp_dir("bundled-init");
    let workspace = root.join("workspace");
    let output = run_primer(
        &root,
        &[
            "init",
            "operating-system",
            "--tool",
            "codex",
            "--path",
            workspace.to_str().expect("workspace path should be utf-8"),
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(workspace.join("AGENTS.md").exists());
    assert!(
        workspace
            .join(".agents/skills/primer-build/SKILL.md")
            .exists()
    );
    assert!(
        workspace
            .join(".primer/recipes/operating-system/recipe.yaml")
            .exists()
    );

    let context =
        fs::read_to_string(workspace.join("AGENTS.md")).expect("failed to read AGENTS.md");
    let recipe_snapshot = workspace
        .join(".primer")
        .join("recipes")
        .join("operating-system");
    assert!(context.contains("schema_version: 2"));
    assert!(context.contains("kind: recipe"));
    assert!(context.contains("id: operating-system"));
    assert!(context.contains(&format!("path: {}", recipe_snapshot.display())));
}
