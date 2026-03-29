use anyhow::{Context, Result, bail};
use comfy_table::Color;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::BuildArgs;
use crate::intent;
use crate::state;
use crate::ui;
use crate::workflow;

struct BuildData {
    source_kind: String,
    source_id: String,
    milestone_id: String,
    milestone_title: String,
    track: String,
    contract_file: PathBuf,
    agent_file: PathBuf,
    intent_file: Option<PathBuf>,
    workspace: PathBuf,
    contract_markdown: String,
    intent_markdown: Option<String>,
    track_guidance_markdown: String,
}

#[derive(Serialize)]
struct BuildJson {
    source: BuildSourceJson,
    current_milestone: BuildMilestoneJson,
    track: String,
    contract_file: String,
    agent_file: String,
    intent_file: Option<String>,
    workspace: String,
    contract_markdown: String,
    intent_markdown: Option<String>,
    track_guidance_markdown: String,
    next_steps: Vec<String>,
}

#[derive(Serialize)]
struct BuildSourceJson {
    kind: String,
    id: String,
}

#[derive(Serialize)]
struct BuildMilestoneJson {
    id: String,
    title: String,
}

pub fn run(workspace_hint: &Path, args: BuildArgs) -> Result<()> {
    let data = collect_build_data(workspace_hint)?;

    if args.json {
        let json = serde_json::to_string_pretty(&BuildJson::from_data(&data))
            .context("failed to serialize build output")?;
        println!("{json}");
        return Ok(());
    }

    render_human(&data);
    Ok(())
}

fn collect_build_data(workspace_hint: &Path) -> Result<BuildData> {
    let state = state::load_from_workspace(workspace_hint)?;
    let workflow = workflow::load(&state.source)?;
    let milestone = workflow::resolve_initial_milestone(&workflow, Some(&state.milestone_id))?;
    let intent = intent::load_for_workflow(&workflow)?;

    let milestone_dir = workflow.path.join("milestones").join(&milestone.id);
    let spec_path = milestone_dir.join("spec.md");
    let agent_path = milestone_dir.join("agent.md");

    if !spec_path.is_file() {
        bail!("milestone contract not found at {}", spec_path.display());
    }
    if !agent_path.is_file() {
        bail!(
            "milestone agent instructions not found at {}",
            agent_path.display()
        );
    }

    let spec = fs::read_to_string(&spec_path)
        .with_context(|| format!("failed to read {}", spec_path.display()))?;
    let agent = fs::read_to_string(&agent_path)
        .with_context(|| format!("failed to read {}", agent_path.display()))?;
    let track_instructions = extract_track_section(&agent, &state.track)?;

    Ok(BuildData {
        source_kind: state.source.kind.as_str().to_string(),
        source_id: state.source.id,
        milestone_id: milestone.id.clone(),
        milestone_title: milestone.title.clone(),
        track: state.track,
        contract_file: spec_path,
        agent_file: agent_path,
        intent_file: intent.as_ref().map(|doc| doc.path.clone()),
        workspace: state.workspace_root,
        contract_markdown: spec,
        intent_markdown: intent.as_ref().map(|doc| doc.markdown.clone()),
        track_guidance_markdown: track_instructions.to_string(),
    })
}

fn render_human(data: &BuildData) {
    ui::section("Primer build");
    println!();
    let mut rows = vec![
        ui::KeyValueRow {
            key: source_label(&data.source_kind).to_string(),
            value: data.source_id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{} ({})", data.milestone_id, data.milestone_title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: data.track.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Contract file".to_string(),
            value: data.contract_file.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
        ui::KeyValueRow {
            key: "Agent file".to_string(),
            value: data.agent_file.display().to_string(),
            value_color: Some(Color::DarkGrey),
        },
    ];
    if let Some(intent_file) = data.intent_file.as_ref() {
        rows.push(ui::KeyValueRow {
            key: "Intent file".to_string(),
            value: intent_file.display().to_string(),
            value_color: Some(Color::DarkGrey),
        });
    }
    rows.push(ui::KeyValueRow {
        key: "Workspace".to_string(),
        value: data.workspace.display().to_string(),
        value_color: Some(Color::Cyan),
    });
    ui::key_value_table(&rows);

    println!();
    ui::section("Milestone contract");
    println!();
    ui::print_markdown(data.contract_markdown.trim_end());

    if let Some(intent_markdown) = data.intent_markdown.as_deref() {
        println!();
        ui::section("Workstream intent");
        println!();
        ui::print_markdown(intent_markdown.trim_end());
    }

    println!();
    ui::section("Track guidance");
    println!();
    ui::print_markdown(data.track_guidance_markdown.trim_end());

    println!();
    ui::section("Next");
    ui::numbered_steps(&human_next_steps(data));
}

impl BuildJson {
    fn from_data(data: &BuildData) -> Self {
        BuildJson {
            source: BuildSourceJson {
                kind: data.source_kind.clone(),
                id: data.source_id.clone(),
            },
            current_milestone: BuildMilestoneJson {
                id: data.milestone_id.clone(),
                title: data.milestone_title.clone(),
            },
            track: data.track.clone(),
            contract_file: data.contract_file.display().to_string(),
            agent_file: data.agent_file.display().to_string(),
            intent_file: data
                .intent_file
                .as_ref()
                .map(|path| path.display().to_string()),
            workspace: data.workspace.display().to_string(),
            contract_markdown: data.contract_markdown.clone(),
            intent_markdown: data.intent_markdown.clone(),
            track_guidance_markdown: data.track_guidance_markdown.clone(),
            next_steps: json_next_steps(data),
        }
    }
}

fn human_next_steps(data: &BuildData) -> Vec<String> {
    vec![
        format!(
            "Implement only {} in {}",
            ui::code(&data.milestone_id),
            ui::code(data.workspace.display().to_string())
        ),
        format!(
            "Run the {} when the milestone is complete",
            ui::reference("skill", "primer-verify")
        ),
        format!(
            "Switch tracks any time with {} or {} if you want different guidance",
            ui::code("primer track learner"),
            ui::code("primer track builder")
        ),
    ]
}

fn json_next_steps(data: &BuildData) -> Vec<String> {
    vec![
        format!(
            "Implement only {} in {}",
            data.milestone_id,
            data.workspace.display()
        ),
        "Run primer verify when the milestone is complete".to_string(),
        "Switch tracks any time with primer track learner or primer track builder if you want different guidance"
            .to_string(),
    ]
}

fn source_label(kind: &str) -> &'static str {
    match kind {
        "recipe" => "Recipe",
        "workstream" => "Workstream",
        _ => "Source",
    }
}

fn extract_track_section<'a>(agent_markdown: &'a str, track: &str) -> Result<&'a str> {
    let heading = match track {
        "learner" => "## Learner Track",
        "builder" => "## Builder Track",
        other => bail!("unsupported track '{}' in workspace state", other),
    };

    let start = agent_markdown
        .find(heading)
        .ok_or_else(|| anyhow::anyhow!("missing '{}' section in agent instructions", heading))?;
    let rest = &agent_markdown[start..];
    let end = rest
        .match_indices("\n## ")
        .map(|(index, _)| index)
        .find(|index| *index > 0)
        .unwrap_or(rest.len());
    Ok(rest[..end].trim())
}
