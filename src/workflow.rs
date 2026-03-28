use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;
use crate::recipe::{self, Milestone};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowSourceKind {
    Recipe,
    Workstream,
}

impl WorkflowSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            WorkflowSourceKind::Recipe => "recipe",
            WorkflowSourceKind::Workstream => "workstream",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WorkflowSourceKind::Recipe => "Recipe",
            WorkflowSourceKind::Workstream => "Workstream",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowSourceRef {
    pub kind: WorkflowSourceKind,
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub source: WorkflowSourceRef,
    pub title: String,
    pub path: PathBuf,
    pub stack_id: Option<String>,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Deserialize)]
struct WorkstreamDoc {
    id: String,
    title: String,
    milestones: Vec<WorkstreamMilestoneDoc>,
}

#[derive(Debug, Deserialize)]
struct WorkstreamMilestoneDoc {
    id: String,
    title: String,
    #[serde(default)]
    prerequisites: Vec<String>,
    goal: Option<String>,
    verification_summary: Option<String>,
    #[serde(default)]
    expected_artifacts: Vec<String>,
    estimated_verify_minutes: Option<u32>,
    split_if_stuck: Option<String>,
}

pub fn load(source: &WorkflowSourceRef) -> Result<Workflow> {
    match source.kind {
        WorkflowSourceKind::Recipe => load_recipe(source),
        WorkflowSourceKind::Workstream => load_workstream(source),
    }
}

pub fn load_from_path(path: &Path) -> Result<Workflow> {
    let path = paths::absolute(path)?;

    if path.join("recipe.yaml").is_file() {
        let recipe = recipe::load_from_path(&path)?;
        return Ok(Workflow {
            source: WorkflowSourceRef {
                kind: WorkflowSourceKind::Recipe,
                id: recipe.id.clone(),
                path: recipe.path.clone(),
            },
            title: recipe.title,
            path: recipe.path,
            stack_id: Some(recipe.stack_id),
            milestones: recipe.milestones,
        });
    }

    if path.join("workstream.yaml").is_file() {
        let id = load_workstream_id(&path)?;
        return load(&WorkflowSourceRef {
            kind: WorkflowSourceKind::Workstream,
            id,
            path,
        });
    }

    bail!(
        "no workflow definition found in {}; expected recipe.yaml or workstream.yaml",
        path.display()
    )
}

pub fn resolve_initial_milestone<'a>(
    workflow: &'a Workflow,
    requested: Option<&str>,
) -> Result<&'a Milestone> {
    if let Some(requested) = requested {
        return workflow
            .milestones
            .iter()
            .find(|milestone| milestone.id == requested)
            .ok_or_else(|| {
                anyhow!(
                    "milestone '{}' is not declared for {} '{}'",
                    requested,
                    workflow.source.kind.as_str(),
                    workflow.source.id
                )
            });
    }

    workflow.milestones.first().ok_or_else(|| {
        anyhow!(
            "{} '{}' declares no milestones",
            workflow.source.kind.as_str(),
            workflow.source.id
        )
    })
}

pub fn milestone_index(workflow: &Workflow, milestone_id: &str) -> Result<usize> {
    workflow
        .milestones
        .iter()
        .position(|milestone| milestone.id == milestone_id)
        .ok_or_else(|| {
            anyhow!(
                "milestone '{}' is not declared for {} '{}'",
                milestone_id,
                workflow.source.kind.as_str(),
                workflow.source.id
            )
        })
}

fn load_recipe(source: &WorkflowSourceRef) -> Result<Workflow> {
    let recipe = recipe::load_from_path(&source.path)?;
    if recipe.id != source.id {
        bail!(
            "recipe id mismatch: state points at '{}', recipe declares '{}'",
            source.id,
            recipe.id
        );
    }

    Ok(Workflow {
        source: source.clone(),
        title: recipe.title,
        path: recipe.path,
        stack_id: Some(recipe.stack_id),
        milestones: recipe.milestones,
    })
}

fn load_workstream(source: &WorkflowSourceRef) -> Result<Workflow> {
    let workstream_yaml = source.path.join("workstream.yaml");
    let raw = fs::read_to_string(&workstream_yaml)
        .with_context(|| format!("failed to read {}", workstream_yaml.display()))?;
    let parsed: WorkstreamDoc = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse {}", workstream_yaml.display()))?;

    if parsed.id != source.id {
        bail!(
            "workstream id mismatch: state points at '{}', workstream.yaml declares '{}'",
            source.id,
            parsed.id
        );
    }

    Ok(Workflow {
        source: source.clone(),
        title: parsed.title,
        path: source.path.clone(),
        stack_id: None,
        milestones: parsed
            .milestones
            .into_iter()
            .map(|milestone| Milestone {
                id: milestone.id,
                title: milestone.title,
                prerequisites: milestone.prerequisites,
                goal: milestone.goal,
                verification_summary: milestone.verification_summary,
                expected_artifacts: milestone.expected_artifacts,
                estimated_verify_minutes: milestone.estimated_verify_minutes,
                split_if_stuck: milestone.split_if_stuck,
            })
            .collect(),
    })
}

fn load_workstream_id(path: &Path) -> Result<String> {
    let workstream_yaml = path.join("workstream.yaml");
    let raw = fs::read_to_string(&workstream_yaml)
        .with_context(|| format!("failed to read {}", workstream_yaml.display()))?;
    let parsed: WorkstreamDoc = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse {}", workstream_yaml.display()))?;
    Ok(parsed.id)
}
