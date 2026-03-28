use anyhow::{Context, Result, bail};
use comfy_table::Color;
use std::fs;
use std::path::Path;

use crate::adapter;
use crate::cli::WorkstreamInitArgs;
use crate::ui;
use crate::workflow::{self, WorkflowSourceKind, WorkflowSourceRef};
use crate::workstream;

pub fn init(workspace_hint: &Path, args: WorkstreamInitArgs) -> Result<()> {
    let repo_root = workstream::ensure_repository_root(workspace_hint)?;
    ensure_no_active_primer_state(&repo_root)?;
    ensure_target_context_is_safe(&repo_root, args.tool)?;

    let spinner = ui::spinner("Scaffolding Primer workstream...");
    let workstream_dir = workstream::scaffold(&repo_root, &args.workstream_id, &args.goal)?;
    let workflow = workflow::load(&WorkflowSourceRef {
        kind: WorkflowSourceKind::Workstream,
        id: args.workstream_id.clone(),
        path: workstream_dir.clone(),
    })?;
    adapter::generate_workstream(
        &workflow,
        &repo_root,
        args.tool,
        args.track.as_str(),
        workstream::INITIAL_MILESTONE_ID,
    )?;
    spinner.finish_and_clear();

    let context_path = repo_root.join(adapter::context_path_for_tool(args.tool));
    ui::success("Initialized Primer workstream");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Repository".to_string(),
            value: repo_root.display().to_string(),
            value_color: Some(Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Workstream".to_string(),
            value: workflow.source.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Goal".to_string(),
            value: args.goal.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Tool".to_string(),
            value: args.tool.display_name().to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: args.track.as_str().to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!(
                "{} ({})",
                workstream::INITIAL_MILESTONE_ID,
                workstream::INITIAL_MILESTONE_TITLE
            ),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "State file".to_string(),
            value: context_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);

    println!();
    ui::section("Next");
    ui::numbered_steps(&[
        format!(
            "Run the {} to review the scaffolded first milestone",
            ui::reference("skill", "primer-build")
        ),
        format!(
            "Replace {} with a real verification script for the first repo-specific milestone",
            ui::code(
                workstream_dir
                    .join("milestones")
                    .join(workstream::INITIAL_MILESTONE_ID)
                    .join("tests")
                    .join("verify.sh")
                    .display()
                    .to_string()
            )
        ),
        format!(
            "Run the {} after you turn the scaffold into a real first milestone",
            ui::reference("skill", "primer-verify")
        ),
    ]);

    Ok(())
}

fn ensure_no_active_primer_state(repo_root: &Path) -> Result<()> {
    for filename in ["CLAUDE.md", "AGENTS.md", "GEMINI.md"] {
        let path = repo_root.join(filename);
        if !path.is_file() {
            continue;
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if text.contains("primer_state:") {
            bail!(
                "Primer workspace state already exists in {}; brownfield v1 supports one active workstream only",
                path.display()
            );
        }
    }

    Ok(())
}

fn ensure_target_context_is_safe(repo_root: &Path, tool: crate::cli::Tool) -> Result<()> {
    let context_path = repo_root.join(adapter::context_path_for_tool(tool));
    if !context_path.exists() {
        return Ok(());
    }

    bail!(
        "refusing to overwrite existing {}; choose a different tool context or move the file first",
        context_path.display()
    )
}
