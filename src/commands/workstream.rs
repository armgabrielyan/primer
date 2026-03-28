use anyhow::{Context, Result, bail};
use comfy_table::Color;
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::adapter;
use crate::cli::{Tool, WorkstreamInitArgs, WorkstreamListArgs, WorkstreamSwitchArgs};
use crate::state::{self, PrimerState};
use crate::ui;
use crate::workflow::{self, WorkflowSourceKind};
use crate::workstream;
use crate::workstream_resume;

#[derive(Debug, Serialize)]
struct WorkstreamListResponse {
    repository: String,
    active_workstream_id: Option<String>,
    count: usize,
    workstreams: Vec<WorkstreamListItem>,
}

#[derive(Debug, Serialize)]
struct WorkstreamListItem {
    id: String,
    title: String,
    active: bool,
    current_milestone_id: String,
    current_milestone_title: String,
    verified: bool,
    milestone_count: usize,
    path: String,
}

pub fn list(workspace_hint: &Path, args: WorkstreamListArgs) -> Result<()> {
    let repo_root = workstream::ensure_repository_root(workspace_hint)?;
    let workstreams = collect_workstream_list(&repo_root)?;

    if args.json {
        let json = serde_json::to_string_pretty(&workstreams)
            .context("failed to serialize workstream list output")?;
        println!("{json}");
        return Ok(());
    }

    ui::section("Primer workstreams");
    println!();

    if workstreams.workstreams.is_empty() {
        ui::info(&format!(
            "No repository-local Primer workstreams found in {}",
            ui::code(&workstreams.repository)
        ));
        println!();
        ui::section("Next");
        ui::numbered_steps(&[format!(
            "Run {} to initialize the first workstream for this repository",
            ui::code("primer workstream init <workstream-id> --goal ... --tool ...")
        )]);
        return Ok(());
    }

    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Repository".to_string(),
            value: repo_root.display().to_string(),
            value_color: Some(Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Active workstream".to_string(),
            value: workstreams
                .active_workstream_id
                .as_deref()
                .unwrap_or("none")
                .to_string(),
            value_color: workstreams
                .active_workstream_id
                .as_ref()
                .map(|_| Color::Green),
        },
        ui::KeyValueRow {
            key: "Count".to_string(),
            value: workstreams.count.to_string(),
            value_color: None,
        },
    ]);

    println!();
    let rows = workstreams
        .workstreams
        .iter()
        .map(|workstream| ui::WorkstreamRow {
            id: workstream.id.clone(),
            title: workstream.title.clone(),
            status: if workstream.active {
                "active".to_string()
            } else {
                "available".to_string()
            },
            status_color: if workstream.active {
                Color::Green
            } else {
                Color::White
            },
            current_milestone: workstream.current_milestone_id.clone(),
            verified: if workstream.verified {
                "yes".to_string()
            } else {
                "no".to_string()
            },
            verified_color: if workstream.verified {
                Color::Green
            } else {
                Color::Yellow
            },
            milestones: workstream.milestone_count.to_string(),
            location: workstream.path.clone(),
        })
        .collect::<Vec<_>>();
    ui::display_workstream_table(&rows);

    println!();
    ui::section("Next");
    let mut steps = Vec::new();
    if let Some(active_workstream_id) = workstreams.active_workstream_id.as_deref() {
        steps.push(format!(
            "Use {} to activate a different workstream",
            ui::code("primer workstream switch <workstream-id>")
        ));
        steps.push(format!(
            "Run the {} to inspect the active milestone for {}",
            ui::reference("skill", "primer-status"),
            ui::code(active_workstream_id)
        ));
    } else {
        steps.push(
            "Primer found workstreams but none is currently active in the root adapter context."
                .to_string(),
        );
        steps.push(
            "If you expected one to be active, restore the repository root Primer context or initialize a workstream again."
                .to_string(),
        );
    }
    ui::numbered_steps(&steps);

    Ok(())
}

fn collect_workstream_list(repo_root: &Path) -> Result<WorkstreamListResponse> {
    let active_state = active_primer_state(repo_root)?;
    let active_workstream_id = active_state
        .as_ref()
        .filter(|state| state.source.kind == WorkflowSourceKind::Workstream)
        .map(|state| state.source.id.clone());
    let workstream_sources = workstream::discover(repo_root)?;

    let workstreams = workstream_sources
        .into_iter()
        .map(|source| {
            let workflow = workflow::load(&source)?;
            let resumed_state = workstream_resume::resolve_for_workflow(&workflow, repo_root)?;
            let current_milestone =
                workflow::resolve_initial_milestone(&workflow, Some(&resumed_state.milestone_id))?;
            let current_milestone_id = current_milestone.id.clone();
            let current_milestone_title = current_milestone.title.clone();
            let is_active = active_workstream_id.as_deref() == Some(source.id.as_str());

            Ok(WorkstreamListItem {
                id: source.id,
                title: workflow.title,
                active: is_active,
                current_milestone_id,
                current_milestone_title,
                verified: resumed_state.verified_milestone_id.is_some(),
                milestone_count: workflow.milestones.len(),
                path: workflow.path.display().to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(WorkstreamListResponse {
        repository: repo_root.display().to_string(),
        active_workstream_id,
        count: workstreams.len(),
        workstreams,
    })
}

pub fn init(workspace_hint: &Path, args: WorkstreamInitArgs) -> Result<()> {
    let repo_root = workstream::ensure_repository_root(workspace_hint)?;
    let active_state = active_primer_state(&repo_root)?;

    if let Some(active_state) = active_state.as_ref() {
        if active_state.source.kind != WorkflowSourceKind::Workstream {
            bail!(
                "Primer is already active in {} with a recipe-backed source; brownfield workstream init only supports repositories with no active Primer state or an existing workstream state",
                active_state.context_path.display()
            );
        }
        ensure_requested_tool_matches_active_context(&repo_root, active_state, args.tool)?;
    } else {
        ensure_target_context_is_safe(&repo_root, args.tool)?;
    }

    let spinner = ui::spinner("Scaffolding Primer workstream...");
    let result = (|| -> Result<(workflow::Workflow, Tool)> {
        workstream::scaffold(&repo_root, &args.workstream_id, &args.goal)?;
        let workflow = workflow::load(&workstream::source_ref(&repo_root, &args.workstream_id))?;
        let tool = match active_state.as_ref() {
            Some(active_state) => adapter::detect_tool(&repo_root, &active_state.context_path)?,
            None => args.tool,
        };
        adapter::generate_workstream(
            &workflow,
            &repo_root,
            tool,
            args.track.as_str(),
            workstream::INITIAL_MILESTONE_ID,
            None,
        )?;
        Ok((workflow, tool))
    })();
    spinner.finish_and_clear();
    let (workflow, tool) = result?;
    let active_state = state::load_from_workspace(&repo_root)?;
    workstream_resume::sync_from_state(&active_state)?;

    let context_path = repo_root.join(adapter::context_path_for_tool(tool));
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
            value: tool.display_name().to_string(),
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
                workflow
                    .path
                    .join("milestones")
                    .join(workstream::INITIAL_MILESTONE_ID)
                    .join("tests")
                    .join("verify.sh")
                    .display()
                    .to_string()
            )
        ),
        format!(
            "Use {} later if you want to move back to another initialized workstream",
            ui::code("primer workstream switch <workstream-id>")
        ),
    ]);

    Ok(())
}

pub fn switch(workspace_hint: &Path, args: WorkstreamSwitchArgs) -> Result<()> {
    let repo_root = workstream::ensure_repository_root(workspace_hint)?;
    let active_state = active_primer_state(&repo_root)?.ok_or_else(|| {
        anyhow::anyhow!(
            "no active Primer workstream found in {}; run `primer workstream init <id> --goal ... --tool ...` first",
            repo_root.display()
        )
    })?;
    if active_state.source.kind != WorkflowSourceKind::Workstream {
        bail!(
            "active Primer state in {} is recipe-backed; workstream switching only applies to repository-local workstreams",
            active_state.context_path.display()
        );
    }

    let target_source = resolve_workstream_source(&repo_root, &args.workstream_id)?;
    if active_state.source.id == target_source.id {
        ui::section("Primer workstream switch");
        println!();
        ui::info(&format!(
            "Workstream {} is already active",
            ui::code(&target_source.id)
        ));
        return Ok(());
    }

    let target_workflow = workflow::load(&target_source)?;
    let resumed_state = workstream_resume::resolve_for_workflow(&target_workflow, &repo_root)?;
    let current_milestone =
        workflow::resolve_initial_milestone(&target_workflow, Some(&resumed_state.milestone_id))?;
    let tool = adapter::detect_tool(&repo_root, &active_state.context_path)?;
    adapter::generate_workstream(
        &target_workflow,
        &repo_root,
        tool,
        &active_state.track,
        &resumed_state.milestone_id,
        resumed_state.verified_milestone_id.as_deref(),
    )?;
    let target_state = state::load_from_workspace(&repo_root)?;
    workstream_resume::sync_from_state(&target_state)?;
    let current_milestone_verified =
        resumed_state.verified_milestone_id.as_deref() == Some(current_milestone.id.as_str());

    ui::section("Primer workstream switch");
    println!();
    ui::success(&format!("Switched to {}", target_workflow.source.id));
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Repository".to_string(),
            value: repo_root.display().to_string(),
            value_color: Some(Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Previous workstream".to_string(),
            value: active_state.source.id,
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Active workstream".to_string(),
            value: target_workflow.source.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: active_state.track.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{} ({})", current_milestone.id, current_milestone.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Verified current milestone".to_string(),
            value: if current_milestone_verified {
                "yes".to_string()
            } else {
                "no".to_string()
            },
            value_color: Some(if current_milestone_verified {
                Color::Green
            } else {
                Color::Yellow
            }),
        },
        ui::KeyValueRow {
            key: "Resume state".to_string(),
            value: if resumed_state.resumed_previous_progress {
                "restored saved progress".to_string()
            } else {
                "started from the first milestone".to_string()
            },
            value_color: None,
        },
        ui::KeyValueRow {
            key: "State file".to_string(),
            value: active_state.context_path.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ]);

    println!();
    ui::section("Next");
    let mut steps = vec![format!(
        "Run the {} to inspect the active milestone for {}",
        ui::reference("skill", "primer-status"),
        ui::code(&target_workflow.source.id)
    )];
    if current_milestone_verified {
        steps.push(format!(
            "Run the {} when you are ready to advance from {}",
            ui::reference("skill", "primer-next-milestone"),
            ui::code(&current_milestone.id)
        ));
    } else {
        steps.push(format!(
            "Run the {} to continue work on {}",
            ui::reference("skill", "primer-build"),
            ui::code(&current_milestone.id)
        ));
    }
    ui::numbered_steps(&steps);

    Ok(())
}

fn active_primer_state(repo_root: &Path) -> Result<Option<PrimerState>> {
    for filename in ["CLAUDE.md", "AGENTS.md", "GEMINI.md"] {
        let path = repo_root.join(filename);
        if !path.is_file() {
            continue;
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if text.contains("primer_state:") {
            return state::load_from_workspace(repo_root).map(Some);
        }
    }

    Ok(None)
}

fn ensure_requested_tool_matches_active_context(
    repo_root: &Path,
    active_state: &PrimerState,
    requested_tool: Tool,
) -> Result<()> {
    let active_tool = adapter::detect_tool(repo_root, &active_state.context_path)?;
    if active_tool != requested_tool {
        bail!(
            "Primer is already active in {} using {}; initializing another workstream currently requires the same tool context",
            active_state.context_path.display(),
            active_tool.display_name()
        );
    }
    Ok(())
}

fn ensure_target_context_is_safe(repo_root: &Path, tool: Tool) -> Result<()> {
    let context_path = repo_root.join(adapter::context_path_for_tool(tool));
    if !context_path.exists() {
        return Ok(());
    }

    bail!(
        "refusing to overwrite existing {}; choose a different tool context or move the file first",
        context_path.display()
    )
}

fn resolve_workstream_source(
    repo_root: &Path,
    workstream_id: &str,
) -> Result<crate::workflow::WorkflowSourceRef> {
    let workstreams = workstream::discover(repo_root)?;
    if let Some(source) = workstreams
        .into_iter()
        .find(|source| source.id == workstream_id)
    {
        return Ok(source);
    }

    let available = workstream::discover(repo_root)?
        .into_iter()
        .map(|source| source.id)
        .collect::<Vec<_>>();
    if available.is_empty() {
        bail!(
            "no Primer workstreams exist in {}; run `primer workstream init <id> --goal ... --tool ...` first",
            repo_root.display()
        );
    }

    bail!(
        "workstream '{}' was not found in {}; available workstreams: {}",
        workstream_id,
        repo_root.join(".primer/workstreams").display(),
        available.join(", ")
    )
}
