use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

pub const INITIAL_MILESTONE_ID: &str = "01-customize-first-milestone";
pub const INITIAL_MILESTONE_TITLE: &str = "Customize the first repo-specific milestone";

#[derive(Serialize)]
struct WorkstreamDoc<'a> {
    id: &'a str,
    title: &'a str,
    goal: &'a str,
    repository_root: PathBuf,
    tracks: WorkstreamTracks<'a>,
    milestones: Vec<WorkstreamMilestone<'a>>,
}

#[derive(Serialize)]
struct WorkstreamTracks<'a> {
    learner: WorkstreamTrack<'a>,
    builder: WorkstreamTrack<'a>,
}

#[derive(Serialize)]
struct WorkstreamTrack<'a> {
    description: &'a str,
}

#[derive(Serialize)]
struct WorkstreamMilestone<'a> {
    id: &'a str,
    title: &'a str,
    goal: &'a str,
    verification_summary: &'a str,
}

pub fn ensure_repository_root(dir: &Path) -> Result<PathBuf> {
    let repo_root = paths::absolute(dir)?;
    let git_entry = repo_root.join(".git");
    if !git_entry.exists() {
        bail!(
            "{} does not look like a repository root; expected a .git entry in the current directory",
            repo_root.display()
        );
    }
    Ok(repo_root)
}

pub fn scaffold(repo_root: &Path, workstream_id: &str, goal: &str) -> Result<PathBuf> {
    let workstream_dir = repo_root
        .join(".primer")
        .join("workstreams")
        .join(workstream_id);
    if workstream_dir.exists() {
        bail!(
            "workstream '{}' already exists at {}",
            workstream_id,
            workstream_dir.display()
        );
    }

    let milestone_dir = workstream_dir.join("milestones").join(INITIAL_MILESTONE_ID);
    let tests_dir = milestone_dir.join("tests");
    fs::create_dir_all(&tests_dir)
        .with_context(|| format!("failed to create {}", tests_dir.display()))?;

    let workstream_title = title_from_id(workstream_id);
    fs::write(
        workstream_dir.join("workstream.yaml"),
        render_workstream_yaml(repo_root, workstream_id, &workstream_title, goal),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            workstream_dir.join("workstream.yaml").display()
        )
    })?;
    fs::write(
        milestone_dir.join("spec.md"),
        render_spec_md(workstream_id, goal),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            milestone_dir.join("spec.md").display()
        )
    })?;
    fs::write(milestone_dir.join("agent.md"), render_agent_md(goal)).with_context(|| {
        format!(
            "failed to write {}",
            milestone_dir.join("agent.md").display()
        )
    })?;
    fs::write(
        milestone_dir.join("explanation.md"),
        render_explanation_md(goal),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            milestone_dir.join("explanation.md").display()
        )
    })?;
    fs::write(tests_dir.join("verify.sh"), render_verify_sh(workstream_id))
        .with_context(|| format!("failed to write {}", tests_dir.join("verify.sh").display()))?;
    make_executable(&tests_dir.join("verify.sh"))?;

    let runtime_dir = repo_root
        .join(".primer")
        .join("runtime")
        .join("workstreams")
        .join(workstream_id);
    fs::create_dir_all(&runtime_dir)
        .with_context(|| format!("failed to create {}", runtime_dir.display()))?;

    Ok(workstream_dir)
}

fn render_workstream_yaml(
    repo_root: &Path,
    workstream_id: &str,
    workstream_title: &str,
    goal: &str,
) -> String {
    serde_yaml::to_string(&WorkstreamDoc {
        id: workstream_id,
        title: workstream_title,
        goal,
        repository_root: repo_root.to_path_buf(),
        tracks: WorkstreamTracks {
            learner: WorkstreamTrack {
                description: "Understand the change surface before making code changes.",
            },
            builder: WorkstreamTrack {
                description: "Make the smallest safe implementation changes per milestone.",
            },
        },
        milestones: vec![WorkstreamMilestone {
            id: INITIAL_MILESTONE_ID,
            title: INITIAL_MILESTONE_TITLE,
            goal: "Replace the placeholder milestone contract and verification script with the first real repo-specific step.",
            verification_summary:
                "Replace tests/verify.sh with a real verification script and make it pass for the first real milestone.",
        }],
    })
    .expect("workstream scaffold YAML should serialize")
}

fn render_spec_md(workstream_id: &str, goal: &str) -> String {
    format!(
        "# Milestone 01: Customize the first repo-specific milestone\n\n\
## Workstream\n\n\
`{workstream_id}`\n\n\
## Workstream goal\n\n\
{goal}\n\n\
## What to do\n\n\
Turn this scaffold into the first real brownfield milestone for this repository.\n\n\
1. Rewrite this spec so it describes one bounded, verifiable change.\n\
2. Update `agent.md` so learner and builder tracks match the repo and milestone.\n\
3. Update `explanation.md` with the most important debugging or architecture context.\n\
4. Replace `tests/verify.sh` with a real verification script for the milestone.\n\n\
## Constraint\n\n\
Do not advance until the replacement verification script actually proves the first milestone is done.\n"
    )
}

fn render_agent_md(goal: &str) -> String {
    format!(
        "# Agent Instructions: {INITIAL_MILESTONE_ID}\n\n\
## Learner Track\n\n\
Explain the repository area this workstream touches before making code changes.\n\
Ask at least one question about the expected boundary of the first real milestone.\n\
Keep the first milestone small enough to verify quickly.\n\n\
## Builder Track\n\n\
Convert this scaffold into the smallest real milestone that advances the goal: {goal}\n\
Keep the milestone bounded, verifiable, and focused on one clear capability change.\n"
    )
}

fn render_explanation_md(goal: &str) -> String {
    format!(
        "# Explanation: {INITIAL_MILESTONE_ID}\n\n\
This scaffold exists so the repository can enter Primer's milestone loop without pretending the first brownfield step is already well-defined.\n\n\
Use this file to capture the shortest useful context for the first real milestone under the workstream goal:\n\n\
- {goal}\n\
- why this is the right first step\n\
- what to verify before advancing\n"
    )
}

fn render_verify_sh(workstream_id: &str) -> String {
    format!(
        "#!/usr/bin/env bash\n\
set -euo pipefail\n\n\
echo \"TODO: replace .primer/workstreams/{workstream_id}/milestones/{INITIAL_MILESTONE_ID}/tests/verify.sh with a real verification script for the first brownfield milestone.\" >&2\n\
exit 1\n"
    )
}

fn title_from_id(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to set permissions for {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}
