use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

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
    let dir = paths::absolute(dir)?;

    for filename in ["CLAUDE.md", "AGENTS.md", "GEMINI.md"] {
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
            recipe_path: paths::normalize(state.recipe_path),
            workspace_root: paths::normalize(state.workspace_root),
            milestone_id: state.milestone_id,
            verified_milestone_id: state.verified_milestone_id,
            track: state.track,
            stack_id: state.stack_id,
        });
    }

    bail!(
        "no Primer workspace state found in {}; expected CLAUDE.md, AGENTS.md, or GEMINI.md",
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

#[cfg(test)]
mod tests {
    use super::load_from_workspace;
    use crate::paths;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic enough for tests")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("primer-{label}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    #[test]
    fn load_from_workspace_supports_gemini_context() {
        let workspace = temp_dir("state-gemini");
        let recipe_path = workspace.join(".primer/recipes/demo");
        fs::create_dir_all(&recipe_path).expect("failed to create recipe path");
        let recipe_path =
            paths::canonicalize(&recipe_path).expect("failed to canonicalize recipe path");
        let workspace_root =
            paths::canonicalize(&workspace).expect("failed to canonicalize workspace root");

        fs::write(
            workspace.join("GEMINI.md"),
            format!(
                "# Primer\n\n```yaml\nprimer_state:\n  recipe_id: demo\n  recipe_path: {}\n  workspace_root: {}\n  milestone_id: 01-alpha\n  verified_milestone_id: null\n  track: learner\n  stack_id: test-stack\n```\n",
                recipe_path.display(),
                workspace_root.display()
            ),
        )
        .expect("failed to write GEMINI.md");

        let state = load_from_workspace(&workspace).expect("failed to load state");
        let context_path = paths::canonicalize(&workspace.join("GEMINI.md"))
            .expect("failed to canonicalize context");
        assert_eq!(state.context_path, context_path);
        assert_eq!(state.recipe_id, "demo");
        assert_eq!(state.milestone_id, "01-alpha");
    }
}
