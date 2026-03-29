use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::workflow::{Workflow, WorkflowSourceKind};

pub const WORKSTREAM_INTENT_FILENAME: &str = "workstream-intent.md";

#[derive(Debug, Clone)]
pub struct IntentDocument {
    pub path: PathBuf,
    pub markdown: String,
}

pub fn scaffold_workstream(goal: &str) -> String {
    format!(
        "# Workstream Intent\n\n\
Keep this file short. It should preserve the durable boundary for the workstream without replacing milestone contracts.\n\n\
## Goal\n\n\
{goal}\n\n\
## Non-goals\n\n\
- Replace this with a tempting adjacent refactor or cleanup item that should stay out of scope.\n\
- Replace this with a broader repository change that should not be bundled into the first milestones.\n\n\
## Constraints\n\n\
- Keep milestones small enough to verify quickly.\n\
- Prefer explicit verification over implied completion.\n\
- Expand scope only after the current milestone passes verification.\n\n\
## Done When\n\n\
- Replace this with the concrete user-visible outcome the workstream should deliver.\n\
- Replace this with the final safety or verification condition that proves the goal is complete.\n"
    )
}

pub fn load_for_workflow(workflow: &Workflow) -> Result<Option<IntentDocument>> {
    if workflow.source.kind != WorkflowSourceKind::Workstream {
        return Ok(None);
    }

    load_optional(&workflow.path)
}

fn load_optional(dir: &Path) -> Result<Option<IntentDocument>> {
    let path = dir.join(WORKSTREAM_INTENT_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }

    let markdown =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some(IntentDocument { path, markdown }))
}
