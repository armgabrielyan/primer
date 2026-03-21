use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PrimerState {
    pub context_path: PathBuf,
    pub recipe_id: String,
    pub recipe_path: PathBuf,
    pub workspace_root: PathBuf,
    pub milestone_id: String,
    pub verified_milestone_id: Option<String>,
    pub track: String,
    pub stack_id: String,
}

#[derive(Debug, Deserialize)]
struct StateEnvelope {
    primer_state: StateDoc,
}

#[derive(Debug, Deserialize)]
struct StateDoc {
    recipe_id: String,
    recipe_path: PathBuf,
    workspace_root: PathBuf,
    milestone_id: String,
    verified_milestone_id: Option<String>,
    track: String,
    stack_id: String,
}

pub fn load_from_workspace(dir: &Path) -> Result<PrimerState> {
    let dir = absolute_path(dir)?;

    for filename in ["CLAUDE.md", "AGENTS.md"] {
        let path = dir.join(filename);
        if !path.is_file() {
            continue;
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let yaml = extract_yaml_block(&text)
            .ok_or_else(|| anyhow!("no primer_state YAML block found in {}", path.display()))?;
        let envelope: StateEnvelope = serde_yaml::from_str(&yaml)
            .with_context(|| format!("failed to parse primer_state in {}", path.display()))?;
        let state = envelope.primer_state;

        if !state.recipe_path.is_absolute() {
            bail!("recipe_path must be absolute in {}", path.display());
        }
        if !state.workspace_root.is_absolute() {
            bail!("workspace_root must be absolute in {}", path.display());
        }

        return Ok(PrimerState {
            context_path: path,
            recipe_id: state.recipe_id,
            recipe_path: state.recipe_path,
            workspace_root: state.workspace_root,
            milestone_id: state.milestone_id,
            verified_milestone_id: state.verified_milestone_id,
            track: state.track,
            stack_id: state.stack_id,
        });
    }

    bail!(
        "no Primer workspace state found in {}; expected CLAUDE.md or AGENTS.md",
        dir.display()
    )
}

fn extract_yaml_block(text: &str) -> Option<String> {
    let start = text.find("```yaml")?;
    let rest = &text[start + "```yaml".len()..];
    let end = rest.find("```")?;
    Some(rest[..end].trim().to_string())
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .with_context(|| format!("failed to resolve {}", path.display()));
    }

    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    Ok(current_dir.join(path))
}
