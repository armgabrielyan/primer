use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;
use crate::workflow::{WorkflowSourceKind, WorkflowSourceRef};

#[derive(Debug, Clone)]
pub struct PrimerState {
    pub context_path: PathBuf,
    pub source: WorkflowSourceRef,
    pub workspace_root: PathBuf,
    pub milestone_id: String,
    pub verified_milestone_id: Option<String>,
    pub track: String,
    pub stack_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StateEnvelopeV2 {
    primer_state: StateDocV2,
}

#[derive(Debug, Deserialize)]
struct StateEnvelopeAny {
    primer_state: StateDocAny,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StateDocAny {
    V2(StateDocV2),
    Legacy(LegacyStateDoc),
}

#[derive(Debug, Deserialize, Serialize)]
struct StateDocV2 {
    schema_version: u32,
    source: SourceDoc,
    workspace_root: PathBuf,
    milestone_id: String,
    verified_milestone_id: Option<String>,
    track: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stack_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyStateDoc {
    recipe_id: String,
    recipe_path: PathBuf,
    workspace_root: PathBuf,
    milestone_id: String,
    verified_milestone_id: Option<String>,
    track: String,
    stack_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SourceDoc {
    kind: WorkflowSourceKind,
    id: String,
    path: PathBuf,
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
        let Some(yaml) = extract_yaml_block(&text) else {
            continue;
        };
        let envelope: StateEnvelopeAny = serde_yaml::from_str(&yaml)
            .with_context(|| format!("failed to parse primer_state in {}", path.display()))?;
        let state = match envelope.primer_state {
            StateDocAny::V2(state) => primer_state_from_v2(path.clone(), state)?,
            StateDocAny::Legacy(state) => primer_state_from_legacy(path.clone(), state)?,
        };
        return Ok(state);
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

    let yaml = serde_yaml::to_string(&StateEnvelopeV2 {
        primer_state: StateDocV2 {
            schema_version: 2,
            source: SourceDoc {
                kind: state.source.kind,
                id: state.source.id.clone(),
                path: state.source.path.clone(),
            },
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

fn primer_state_from_v2(context_path: PathBuf, state: StateDocV2) -> Result<PrimerState> {
    if state.schema_version != 2 {
        bail!(
            "unsupported primer_state schema_version {} in {}",
            state.schema_version,
            context_path.display()
        );
    }
    if !state.source.path.is_absolute() {
        bail!("source.path must be absolute in {}", context_path.display());
    }
    if !state.workspace_root.is_absolute() {
        bail!(
            "workspace_root must be absolute in {}",
            context_path.display()
        );
    }

    Ok(PrimerState {
        context_path,
        source: WorkflowSourceRef {
            kind: state.source.kind,
            id: state.source.id,
            path: paths::normalize(state.source.path),
        },
        workspace_root: paths::normalize(state.workspace_root),
        milestone_id: state.milestone_id,
        verified_milestone_id: state.verified_milestone_id,
        track: state.track,
        stack_id: state.stack_id,
    })
}

fn primer_state_from_legacy(context_path: PathBuf, state: LegacyStateDoc) -> Result<PrimerState> {
    if !state.recipe_path.is_absolute() {
        bail!("recipe_path must be absolute in {}", context_path.display());
    }
    if !state.workspace_root.is_absolute() {
        bail!(
            "workspace_root must be absolute in {}",
            context_path.display()
        );
    }

    Ok(PrimerState {
        context_path,
        source: WorkflowSourceRef {
            kind: WorkflowSourceKind::Recipe,
            id: state.recipe_id,
            path: paths::normalize(state.recipe_path),
        },
        workspace_root: paths::normalize(state.workspace_root),
        milestone_id: state.milestone_id,
        verified_milestone_id: state.verified_milestone_id,
        track: state.track,
        stack_id: Some(state.stack_id),
    })
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
    use crate::workflow::WorkflowSourceKind;
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
    fn load_from_workspace_supports_legacy_recipe_state() {
        let workspace = temp_dir("state-legacy");
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
        assert_eq!(state.source.kind, WorkflowSourceKind::Recipe);
        assert_eq!(state.source.id, "demo");
        assert_eq!(state.milestone_id, "01-alpha");
        assert_eq!(state.stack_id.as_deref(), Some("test-stack"));
    }

    #[test]
    fn load_from_workspace_supports_v2_workstream_state() {
        let workspace = temp_dir("state-workstream");
        let workstream_path = workspace.join(".primer/workstreams/auth-refactor");
        fs::create_dir_all(&workstream_path).expect("failed to create workstream path");
        let workstream_path =
            paths::canonicalize(&workstream_path).expect("failed to canonicalize workstream path");
        let workspace_root =
            paths::canonicalize(&workspace).expect("failed to canonicalize workspace root");

        fs::write(
            workspace.join("AGENTS.md"),
            format!(
                "# Primer\n\n```yaml\nprimer_state:\n  schema_version: 2\n  source:\n    kind: workstream\n    id: auth-refactor\n    path: {}\n  workspace_root: {}\n  milestone_id: 01-map-boundary\n  verified_milestone_id: null\n  track: builder\n```\n",
                workstream_path.display(),
                workspace_root.display()
            ),
        )
        .expect("failed to write AGENTS.md");

        let state = load_from_workspace(&workspace).expect("failed to load state");
        assert_eq!(state.source.kind, WorkflowSourceKind::Workstream);
        assert_eq!(state.source.id, "auth-refactor");
        assert_eq!(state.track, "builder");
        assert!(state.stack_id.is_none());
    }
}
