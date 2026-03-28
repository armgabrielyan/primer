use anyhow::{Context, Result, bail};
use console::style;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::cli::MilestoneLintArgs;
use crate::paths;
use crate::state;
use crate::ui;
use crate::validation::{self, LintFinding, LintSeverity};
use crate::workflow::{self, Workflow, WorkflowSourceKind};

#[derive(Debug, Serialize)]
struct MilestoneLintResponse {
    source: MilestoneLintSourceJson,
    milestone: MilestoneLintMilestoneJson,
    clean: bool,
    errors: usize,
    warnings: usize,
    advice: usize,
    findings: Vec<LintFinding>,
}

#[derive(Debug, Serialize)]
struct MilestoneLintSourceJson {
    kind: String,
    id: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct MilestoneLintMilestoneJson {
    id: String,
    title: String,
    path: String,
}

struct MilestoneLintTarget {
    workflow: Workflow,
    milestone_id: String,
}

pub fn lint(workspace_hint: &Path, args: MilestoneLintArgs) -> Result<()> {
    let target = resolve_target(workspace_hint, &args)?;
    let milestone =
        workflow::resolve_initial_milestone(&target.workflow, Some(&target.milestone_id))?;
    let milestone_index = workflow::milestone_index(&target.workflow, &target.milestone_id)?;
    let milestone_dir = target.workflow.path.join("milestones").join(&milestone.id);
    let findings = validation::lint_milestone(
        &milestone_dir,
        milestone,
        milestone_index,
        target.workflow.source.kind == WorkflowSourceKind::Recipe,
    );

    let response = MilestoneLintResponse {
        source: MilestoneLintSourceJson {
            kind: target.workflow.source.kind.as_str().to_string(),
            id: target.workflow.source.id.clone(),
            path: target.workflow.path.display().to_string(),
        },
        milestone: MilestoneLintMilestoneJson {
            id: milestone.id.clone(),
            title: milestone.title.clone(),
            path: milestone_dir.display().to_string(),
        },
        clean: findings.is_empty(),
        errors: count_findings(&findings, LintSeverity::Error),
        warnings: count_findings(&findings, LintSeverity::Warning),
        advice: count_findings(&findings, LintSeverity::Advice),
        findings,
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&response)
                .context("failed to serialize milestone lint output")?
        );
    } else {
        print_human_report(&response);
    }

    if response
        .findings
        .iter()
        .any(|finding| finding.severity.blocks())
    {
        bail!(
            "milestone lint found {} error(s) and {} warning(s)",
            response.errors,
            response.warnings
        );
    }

    Ok(())
}

fn resolve_target(workspace_hint: &Path, args: &MilestoneLintArgs) -> Result<MilestoneLintTarget> {
    if let Some(path) = args.path.as_deref() {
        let (workflow_path, inferred_milestone_id) = resolve_workflow_path(path)?;
        let workflow = workflow::load_from_path(&workflow_path)?;
        let milestone_id = explicit_or_default_milestone_id(
            &workflow,
            args.milestone_id
                .as_deref()
                .or(inferred_milestone_id.as_deref()),
        )?;
        return Ok(MilestoneLintTarget {
            workflow,
            milestone_id,
        });
    }

    if let Ok(active_state) = state::load_from_workspace(workspace_hint) {
        let workflow = workflow::load(&active_state.source)?;
        let milestone_id = explicit_or_default_milestone_id(
            &workflow,
            args.milestone_id
                .as_deref()
                .or(Some(&active_state.milestone_id)),
        )?;
        return Ok(MilestoneLintTarget {
            workflow,
            milestone_id,
        });
    }

    if workspace_hint.join("recipe.yaml").is_file()
        || workspace_hint.join("workstream.yaml").is_file()
    {
        let workflow = workflow::load_from_path(workspace_hint)?;
        let milestone_id =
            explicit_or_default_milestone_id(&workflow, args.milestone_id.as_deref())?;
        return Ok(MilestoneLintTarget {
            workflow,
            milestone_id,
        });
    }

    bail!("milestone lint needs an active workspace or a --path to a recipe/workstream directory")
}

fn resolve_workflow_path(path: &Path) -> Result<(PathBuf, Option<String>)> {
    let path = paths::absolute(path)?;

    if path.join("recipe.yaml").is_file() || path.join("workstream.yaml").is_file() {
        return Ok((path, None));
    }

    let Some(milestones_dir) = path.parent() else {
        bail!(
            "{} is not a recipe, workstream, or milestone directory",
            path.display()
        );
    };
    let Some(milestone_id) = path.file_name().and_then(|name| name.to_str()) else {
        bail!(
            "{} is not a recipe, workstream, or milestone directory",
            path.display()
        );
    };
    let Some(milestones_label) = milestones_dir.file_name().and_then(|name| name.to_str()) else {
        bail!(
            "{} is not a recipe, workstream, or milestone directory",
            path.display()
        );
    };

    if milestones_label != "milestones" {
        bail!(
            "{} is not a recipe, workstream, or milestone directory",
            path.display()
        );
    }

    let Some(workflow_path) = milestones_dir.parent() else {
        bail!(
            "{} is not a recipe, workstream, or milestone directory",
            path.display()
        );
    };
    if !workflow_path.join("recipe.yaml").is_file()
        && !workflow_path.join("workstream.yaml").is_file()
    {
        bail!(
            "{} is not a recipe, workstream, or milestone directory",
            path.display()
        );
    }

    Ok((workflow_path.to_path_buf(), Some(milestone_id.to_string())))
}

fn explicit_or_default_milestone_id(
    workflow: &Workflow,
    requested: Option<&str>,
) -> Result<String> {
    if let Some(requested) = requested {
        workflow::resolve_initial_milestone(workflow, Some(requested))?;
        return Ok(requested.to_string());
    }

    workflow
        .milestones
        .first()
        .map(|milestone| milestone.id.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} '{}' declares no milestones",
                workflow.source.kind.as_str(),
                workflow.source.id
            )
        })
}

fn count_findings(findings: &[LintFinding], severity: LintSeverity) -> usize {
    findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .count()
}

fn print_human_report(response: &MilestoneLintResponse) {
    ui::section("Primer milestone lint");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: source_label(&response.source.kind).to_string(),
            value: response.source.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workflow path".to_string(),
            value: response.source.path.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Milestone".to_string(),
            value: format!("{} ({})", response.milestone.id, response.milestone.title),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Milestone path".to_string(),
            value: response.milestone.path.clone(),
            value_color: None,
        },
    ]);

    println!();
    if response.findings.is_empty() {
        ui::success("Milestone lint passed.");
        return;
    }

    ui::section("Findings");
    for finding in &response.findings {
        print_finding(finding);
    }

    println!();
    ui::section("Summary");
    println!(
        "{} error(s), {} warning(s), {} advice item(s).",
        response.errors, response.warnings, response.advice
    );
    if response.errors == 0 && response.warnings == 0 {
        ui::success("Milestone lint passed.");
    }
}

fn print_finding(finding: &LintFinding) {
    let severity = match finding.severity {
        LintSeverity::Error => style(finding.severity.label()).red().bold(),
        LintSeverity::Warning => style(finding.severity.label()).yellow().bold(),
        LintSeverity::Advice => style(finding.severity.label()).blue().bold(),
    };

    println!("{} [{}] {}", severity, finding.code, finding.message);
}

fn source_label(kind: &str) -> &'static str {
    match kind {
        "recipe" => "Recipe",
        "workstream" => "Workstream",
        _ => "Source",
    }
}
