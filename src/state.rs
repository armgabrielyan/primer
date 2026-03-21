use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize, Serialize)]
struct StateEnvelope {
    primer_state: StateDoc,
}

#[derive(Debug, Deserialize, Serialize)]
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

pub fn write(state: &PrimerState) -> Result<()> {
    let path = &state.context_path;
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let (start, end) = yaml_block_span(&text)
        .ok_or_else(|| anyhow!("no primer_state YAML block found in {}", path.display()))?;

    let yaml = serde_yaml::to_string(&StateEnvelope {
        primer_state: StateDoc {
            recipe_id: state.recipe_id.clone(),
            recipe_path: state.recipe_path.clone(),
            workspace_root: state.workspace_root.clone(),
            milestone_id: state.milestone_id.clone(),
            verified_milestone_id: state.verified_milestone_id.clone(),
            track: state.track.clone(),
            stack_id: state.stack_id.clone(),
        },
    })
    .context("failed to serialize primer_state")?;

    let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml);
    let replacement = format!("```yaml\n{}```", yaml);

    let mut updated = String::with_capacity(text.len() + replacement.len());
    updated.push_str(&text[..start]);
    updated.push_str(&replacement);
    updated.push_str(&text[end..]);

    fs::write(path, updated).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn extract_yaml_block(text: &str) -> Option<String> {
    let (start, end) = yaml_block_span(text)?;
    let open_len = "```yaml".len();
    Some(text[start + open_len..end - "```".len()].trim().to_string())
}

fn yaml_block_span(text: &str) -> Option<(usize, usize)> {
    let start = text.find("```yaml")?;
    let rest = &text[start + "```yaml".len()..];
    let end = rest.find("```")?;
    Some((start, start + "```yaml".len() + end + "```".len()))
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .with_context(|| format!("failed to resolve {}", path.display()));
    }

    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    Ok(current_dir.join(path))
}
