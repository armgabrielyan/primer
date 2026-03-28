use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

use crate::bundled;
use crate::cli::Tool;
use crate::recipe::Recipe;
use crate::workflow::{Workflow, WorkflowSourceKind};

const PRIMER_WORKFLOW_FILES: &[&str] = &[
    "build.md",
    "verify.md",
    "track.md",
    "explain.md",
    "status.md",
    "next-milestone.md",
];

struct AdapterWorkflow<'a> {
    title: &'a str,
    source_kind: WorkflowSourceKind,
    source_id: &'a str,
    source_path: &'a Path,
    stack_id: Option<&'a str>,
    milestones: &'a [crate::recipe::Milestone],
}

pub fn generate(
    recipe: &Recipe,
    recipe_path: &Path,
    output_dir: &Path,
    tool: Tool,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let workflow = AdapterWorkflow {
        title: &recipe.title,
        source_kind: WorkflowSourceKind::Recipe,
        source_id: &recipe.id,
        source_path: recipe_path,
        stack_id: Some(&recipe.stack_id),
        milestones: &recipe.milestones,
    };
    generate_internal(&workflow, output_dir, tool, track, milestone_id)
}

pub fn generate_workstream(
    workflow: &Workflow,
    output_dir: &Path,
    tool: Tool,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let workflow = AdapterWorkflow {
        title: &workflow.title,
        source_kind: workflow.source.kind,
        source_id: &workflow.source.id,
        source_path: &workflow.path,
        stack_id: workflow.stack_id.as_deref(),
        milestones: &workflow.milestones,
    };
    generate_internal(&workflow, output_dir, tool, track, milestone_id)
}

pub fn context_path_for_tool(tool: Tool) -> &'static str {
    match tool {
        Tool::Codex | Tool::Cursor | Tool::Opencode => "AGENTS.md",
        Tool::Claude => "CLAUDE.md",
        Tool::Gemini => "GEMINI.md",
    }
}

pub fn detect_tool(workspace_root: &Path, context_path: &Path) -> Result<Tool> {
    match context_path.file_name().and_then(|name| name.to_str()) {
        Some("CLAUDE.md") => Ok(Tool::Claude),
        Some("GEMINI.md") => Ok(Tool::Gemini),
        Some("AGENTS.md") => {
            let mut matches = Vec::new();
            if workspace_root
                .join(".agents/skills/primer-build/SKILL.md")
                .is_file()
            {
                matches.push(Tool::Codex);
            }
            if workspace_root
                .join(".cursor/skills/primer-build/SKILL.md")
                .is_file()
            {
                matches.push(Tool::Cursor);
            }
            if workspace_root
                .join(".opencode/skills/primer-build/SKILL.md")
                .is_file()
            {
                matches.push(Tool::Opencode);
            }

            match matches.as_slice() {
                [tool] => Ok(*tool),
                [] => bail!(
                    "unable to infer Primer tool from {}; expected one of .agents, .cursor, or .opencode Primer skill directories",
                    context_path.display()
                ),
                _ => bail!(
                    "unable to infer Primer tool from {}; multiple Primer AGENTS.md tool directories are present",
                    context_path.display()
                ),
            }
        }
        _ => bail!(
            "unable to infer Primer tool from unsupported context path {}",
            context_path.display()
        ),
    }
}

fn generate_internal(
    workflow: &AdapterWorkflow<'_>,
    output_dir: &Path,
    tool: Tool,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    ensure_valid_initial_milestone(workflow, milestone_id)?;
    match tool {
        Tool::Codex => generate_codex(workflow, output_dir, track, milestone_id),
        Tool::Claude => generate_claude(workflow, output_dir, track, milestone_id),
        Tool::Cursor => generate_cursor(workflow, output_dir, track, milestone_id),
        Tool::Gemini => generate_gemini(workflow, output_dir, track, milestone_id),
        Tool::Opencode => generate_opencode(workflow, output_dir, track, milestone_id),
    }
}

fn ensure_valid_initial_milestone(
    workflow: &AdapterWorkflow<'_>,
    milestone_id: &str,
) -> Result<()> {
    if workflow
        .milestones
        .iter()
        .any(|milestone| milestone.id == milestone_id)
    {
        return Ok(());
    }
    bail!(
        "{} '{}': milestone_id '{}' is not declared in milestones",
        workflow.source_kind.as_str(),
        workflow.source_id,
        milestone_id
    )
}

fn generate_codex(
    workflow: &AdapterWorkflow<'_>,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let skills_root = output_dir.join(".agents").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &agents_md,
        render_adapter_context(workflow, output_dir, track, milestone_id),
    )?;

    for filename in PRIMER_WORKFLOW_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        let agents_dir = skill_dir.join("agents");
        fs::create_dir_all(&agents_dir)?;

        let body = match command_name {
            "verify" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "track" => render_track_skill_body(shared_body),
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_skill_md(&skill_name, &command_description(command_name), &body),
        )?;
        fs::write(
            agents_dir.join("openai.yaml"),
            render_openai_yaml(
                &format!(
                    "{}: {}",
                    workflow.title,
                    title_case(&command_name.replace('-', " "))
                ),
                &format!(
                    "{} for the current Primer workflow",
                    capitalize(command_label(command_name))
                ),
                &default_prompt(command_name),
            ),
        )?;
    }

    Ok(())
}

fn generate_claude(
    workflow: &AdapterWorkflow<'_>,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let claude_md = output_dir.join("CLAUDE.md");
    let commands_dir = output_dir.join(".claude").join("commands");
    fs::create_dir_all(&commands_dir)?;

    fs::write(
        &claude_md,
        render_adapter_context(workflow, output_dir, track, milestone_id),
    )?;

    for filename in PRIMER_WORKFLOW_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let target_name = format!("primer-{command_name}.md");
        let contents = match command_name {
            "verify" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_claude_command(command_name, shared_body)
            }
            "track" => render_track_claude_command(shared_body),
            "build" => render_build_claude_command(shared_body),
            _ => shared_body.to_string(),
        };
        fs::write(commands_dir.join(target_name), contents)?;
    }

    Ok(())
}

fn generate_opencode(
    workflow: &AdapterWorkflow<'_>,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let skills_root = output_dir.join(".opencode").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &agents_md,
        render_adapter_context(workflow, output_dir, track, milestone_id),
    )?;

    for filename in PRIMER_WORKFLOW_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        fs::create_dir_all(&skill_dir)?;

        let body = match command_name {
            "verify" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "track" => render_track_skill_body(shared_body),
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_opencode_skill_md(&skill_name, &command_description(command_name), &body),
        )?;
    }

    Ok(())
}

fn generate_cursor(
    workflow: &AdapterWorkflow<'_>,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let skills_root = output_dir.join(".cursor").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &agents_md,
        render_adapter_context(workflow, output_dir, track, milestone_id),
    )?;

    for filename in PRIMER_WORKFLOW_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        fs::create_dir_all(&skill_dir)?;

        let body = match command_name {
            "verify" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "track" => render_track_skill_body(shared_body),
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_skill_md(&skill_name, &command_description(command_name), &body),
        )?;
    }

    Ok(())
}

fn generate_gemini(
    workflow: &AdapterWorkflow<'_>,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let gemini_md = output_dir.join("GEMINI.md");
    let skills_root = output_dir.join(".gemini").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &gemini_md,
        render_adapter_context(workflow, output_dir, track, milestone_id),
    )?;

    for filename in PRIMER_WORKFLOW_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        fs::create_dir_all(&skill_dir)?;

        let body = match command_name {
            "verify" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "track" => render_track_skill_body(shared_body),
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_skill_md(&skill_name, &command_description(command_name), &body),
        )?;
    }

    Ok(())
}

fn render_adapter_context(
    workflow: &AdapterWorkflow<'_>,
    workspace_root: &Path,
    track: &str,
    milestone_id: &str,
) -> String {
    let stack_id = workflow
        .stack_id
        .map(|stack_id| format!("  stack_id: {stack_id}\n"))
        .unwrap_or_default();
    let source_label = workflow.source_kind.label();
    format!(
        "# Primer — {}\n\n```yaml\nprimer_state:\n  schema_version: 2\n  source:\n    kind: {}\n    id: {}\n    path: {}\n  workspace_root: {}\n  milestone_id: {}\n  verified_milestone_id: null\n  track: {}\n{}```\n\n## {} location\n\n{}/\n\n## Workspace root\n\n{}/\n\n## Rules\n\n- Always read the current milestone `agent.md` before starting work.\n- Work in this project workspace, not in the `primer` repository.\n- Build the current milestone in small steps and do not implement future milestones early.\n- Run current milestone verification before declaring completion.\n- Only run `primer-next-milestone` after `primer-verify` has marked the current milestone as verified.\n- Use the local `primer` CLI as the source of truth for `primer-build`, `primer-track`, `primer-verify`, `primer-status`, `primer-explain`, and `primer-next-milestone`.\n- Use the generated Primer workflow actions for behavior rules.\n\n## Track Invariants\n\n{}\n\n## Available workflow actions\n\n- `primer-build` — implement the current milestone step by step\n- `primer-track` — switch the active learner or builder track for this workspace\n- `primer-next-milestone` — advance state only after the milestone is already verified\n- `primer-verify` — run current milestone verification\n- `primer-explain` — show current milestone explanation\n- `primer-status` — show current state and progress\n",
        workflow.title,
        workflow.source_kind.as_str(),
        workflow.source_id,
        workflow.source_path.display(),
        workspace_root.display(),
        milestone_id,
        track,
        stack_id,
        source_label,
        workflow.source_path.display(),
        workspace_root.display(),
        render_track_invariants(track)
    )
}

fn render_skill_md(skill_name: &str, description: &str, body: &str) -> String {
    format!("---\nname: {skill_name}\ndescription: {description}\n---\n\n{body}\n")
}

fn render_opencode_skill_md(skill_name: &str, description: &str, body: &str) -> String {
    format!(
        "---\nname: {skill_name}\ndescription: {description}\ncompatibility: opencode\n---\n\n{body}\n"
    )
}

fn render_openai_yaml(display_name: &str, short_description: &str, default_prompt: &str) -> String {
    format!(
        "interface:\n  display_name: \"{display_name}\"\n  short_description: \"{short_description}\"\n  default_prompt: \"{default_prompt}\"\n"
    )
}

fn render_cli_backed_skill_body(command_name: &str, shared_body: &str) -> String {
    format!(
        "Use the local Primer CLI as the source of truth for this workflow action.\n\n## Active track rules\n\n- Respect the current `track` from `primer_state`.\n- In learner track, teach in small steps, explain before coding, ask at least one question before major implementation, and pause for understanding at natural checkpoints.\n- In builder track, implement directly, keep commentary minimal, and stay focused on the smallest complete change.\n\n## Required action\n\n1. Run `primer {command_name}` from the current workspace root.\n2. Return the CLI output to the user faithfully.\n3. Do not manually edit `primer_state`; the CLI owns state transitions for this skill.\n4. If the `primer` executable is unavailable, stop and tell the user to install or build the Primer CLI.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_build_skill_body(shared_body: &str) -> String {
    format!(
        "Use the local Primer CLI to load the current milestone spec and track guidance before making changes.\n\n## Active track rules\n\n- Respect the current `track` from `primer_state`.\n- In learner track, use a teacher-student style: explain before coding, move in small steps, ask at least one question before major implementation, and check understanding before advancing.\n- In builder track, implement directly, keep commentary minimal, and focus on the smallest milestone-completing change.\n\n## Required action\n\n1. Run `primer build` from the current workspace root.\n2. Use that output as the current milestone contract and active track guidance.\n3. Then implement only the current milestone scope in the workspace.\n4. Recommend running `primer-verify` when the milestone is complete.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_track_skill_body(shared_body: &str) -> String {
    format!(
        "Use the local Primer CLI to switch the active learner or builder track for the current workspace.\n\n## Required action\n\n1. Infer the requested target track from the user's request.\n2. Run `primer track <learner|builder>` from the current workspace root.\n3. Return the CLI output to the user faithfully.\n4. Do not manually edit `primer_state`; the CLI owns state transitions for this skill.\n5. If the user did not specify a target track clearly, ask a short clarifying question before running the command.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_cli_backed_claude_command(command_name: &str, shared_body: &str) -> String {
    format!(
        "# Primer Skill: `primer-{command_name}`\n\nUse the local Primer CLI as the source of truth for this workflow action.\n\n## Active track rules\n\n- Respect the current `track` from `primer_state`.\n- In learner track, teach in small steps, explain before coding, ask at least one question before major implementation, and pause for understanding at natural checkpoints.\n- In builder track, implement directly, keep commentary minimal, and stay focused on the smallest complete change.\n\n## Required action\n\n1. Run `primer {command_name}` from the current workspace root.\n2. Return the CLI output to the user faithfully.\n3. Do not manually edit `primer_state`; the CLI owns state transitions for this skill.\n4. If the `primer` executable is unavailable, stop and tell the user to install or build the Primer CLI.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_track_claude_command(shared_body: &str) -> String {
    format!(
        "# Primer Skill: `primer-track`\n\nUse the local Primer CLI to switch the active learner or builder track for the current workspace.\n\n## Required action\n\n1. Infer the requested target track from the user's request.\n2. Run `primer track <learner|builder>` from the current workspace root.\n3. Return the CLI output to the user faithfully.\n4. Do not manually edit `primer_state`; the CLI owns state transitions for this skill.\n5. If the user did not specify a target track clearly, ask a short clarifying question before running the command.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_build_claude_command(shared_body: &str) -> String {
    format!(
        "# Primer Skill: `primer-build`\n\nUse the local Primer CLI to load the current milestone spec and track guidance before making changes.\n\n## Active track rules\n\n- Respect the current `track` from `primer_state`.\n- In learner track, use a teacher-student style: explain before coding, move in small steps, ask at least one question before major implementation, and check understanding before advancing.\n- In builder track, implement directly, keep commentary minimal, and focus on the smallest milestone-completing change.\n\n## Required action\n\n1. Run `primer build` from the current workspace root.\n2. Use that output as the current milestone contract and active track guidance.\n3. Then implement only the current milestone scope in the workspace.\n4. Recommend running `primer-verify` when the milestone is complete.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn command_label(command_name: &str) -> &str {
    match command_name {
        "next-milestone" => "next milestone",
        "track" => "switch track",
        other => other,
    }
}

fn command_description(command_name: &str) -> String {
    match command_name {
        "track" => {
            "Use when the user wants to switch the active learner or builder track for the current Primer workspace.".to_string()
        }
        _ => format!(
            "Use when the user wants to {} for the current Primer recipe in this repo workspace.",
            command_name.replace('-', " ")
        ),
    }
}

fn default_prompt(command_name: &str) -> String {
    match command_name {
        "track" => "Use $primer-track to switch the active learner or builder track for the current recipe milestone.".to_string(),
        _ => format!("Use $primer-{command_name} for the current recipe milestone."),
    }
}

fn render_track_invariants(track: &str) -> &'static str {
    match track {
        "learner" => {
            "- Use a teacher-student style by default.\n- Explain the goal of each small step before changing code.\n- Ask at least one question before major implementation work begins.\n- Pause at natural checkpoints and confirm understanding before moving on.\n- Prefer incremental changes over large jumps."
        }
        "builder" => {
            "- Implement directly and keep commentary minimal unless the user asks for more detail.\n- Prefer the smallest complete change that advances the milestone.\n- Stay tightly scoped to the current milestone contract."
        }
        _ => "- Respect the active track from `primer_state`.",
    }
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::generate;
    use crate::cli::Tool;
    use crate::recipe::{self, RecipeSource};
    use std::fs;
    use std::path::{Path, PathBuf};
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

    fn read(path: &Path) -> String {
        fs::read_to_string(path).expect("failed to read test file")
    }

    #[test]
    fn codex_generation_creates_expected_files() {
        let source = RecipeSource::Bundled;
        let recipe = recipe::load_by_id(&source, "operating-system").expect("recipe should load");
        let out = temp_dir("codex-gen");
        let recipe_path = recipe::materialize_into_workspace(&source, "operating-system", &out)
            .expect("recipe materialization should succeed");

        generate(
            &recipe,
            &recipe_path,
            &out,
            Tool::Codex,
            "learner",
            "01-bootloader",
        )
        .expect("adapter generation should succeed");

        assert!(out.join("AGENTS.md").exists());
        assert!(out.join(".agents/skills/primer-build/SKILL.md").exists());
        assert!(out.join(".agents/skills/primer-track/SKILL.md").exists());
        assert!(out.join(".agents/skills/primer-verify/SKILL.md").exists());

        let context = read(&out.join("AGENTS.md"));
        assert!(context.contains("schema_version: 2"));
        assert!(context.contains("kind: recipe"));
        assert!(context.contains("id: operating-system"));
        assert!(context.contains("milestone_id: 01-bootloader"));
        assert!(context.contains("primer-track"));
        assert!(context.contains(&format!("path: {}", recipe_path.display())));
    }

    #[test]
    fn claude_generation_creates_expected_files() {
        let source = RecipeSource::Bundled;
        let recipe = recipe::load_by_id(&source, "operating-system").expect("recipe should load");
        let out = temp_dir("claude-gen");
        let recipe_path = recipe::materialize_into_workspace(&source, "operating-system", &out)
            .expect("recipe materialization should succeed");

        generate(
            &recipe,
            &recipe_path,
            &out,
            Tool::Claude,
            "builder",
            "03-vga-output",
        )
        .expect("adapter generation should succeed");

        assert!(out.join("CLAUDE.md").exists());
        assert!(out.join(".claude/commands/primer-build.md").exists());
        assert!(out.join(".claude/commands/primer-track.md").exists());
        assert!(
            out.join(".claude/commands/primer-next-milestone.md")
                .exists()
        );

        let context = read(&out.join("CLAUDE.md"));
        assert!(context.contains("kind: recipe"));
        assert!(context.contains("id: operating-system"));
        assert!(context.contains("track: builder"));
        assert!(context.contains("milestone_id: 03-vga-output"));
    }

    #[test]
    fn opencode_generation_creates_expected_files() {
        let source = RecipeSource::Bundled;
        let recipe = recipe::load_by_id(&source, "operating-system").expect("recipe should load");
        let out = temp_dir("opencode-gen");
        let recipe_path = recipe::materialize_into_workspace(&source, "operating-system", &out)
            .expect("recipe materialization should succeed");

        generate(
            &recipe,
            &recipe_path,
            &out,
            Tool::Opencode,
            "learner",
            "01-bootloader",
        )
        .expect("adapter generation should succeed");

        assert!(out.join("AGENTS.md").exists());
        assert!(out.join(".opencode/skills/primer-build/SKILL.md").exists());
        assert!(out.join(".opencode/skills/primer-track/SKILL.md").exists());
        assert!(out.join(".opencode/skills/primer-verify/SKILL.md").exists());

        let context = read(&out.join("AGENTS.md"));
        assert!(context.contains("kind: recipe"));
        assert!(context.contains("id: operating-system"));
        assert!(context.contains("milestone_id: 01-bootloader"));

        let skill = read(&out.join(".opencode/skills/primer-build/SKILL.md"));
        assert!(skill.contains("name: primer-build"));
        assert!(skill.contains("compatibility: opencode"));
    }

    #[test]
    fn gemini_generation_creates_expected_files() {
        let source = RecipeSource::Bundled;
        let recipe = recipe::load_by_id(&source, "operating-system").expect("recipe should load");
        let out = temp_dir("gemini-gen");
        let recipe_path = recipe::materialize_into_workspace(&source, "operating-system", &out)
            .expect("recipe materialization should succeed");

        generate(
            &recipe,
            &recipe_path,
            &out,
            Tool::Gemini,
            "learner",
            "01-bootloader",
        )
        .expect("adapter generation should succeed");

        assert!(out.join("GEMINI.md").exists());
        assert!(out.join(".gemini/skills/primer-build/SKILL.md").exists());
        assert!(out.join(".gemini/skills/primer-track/SKILL.md").exists());
        assert!(out.join(".gemini/skills/primer-verify/SKILL.md").exists());

        let context = read(&out.join("GEMINI.md"));
        assert!(context.contains("kind: recipe"));
        assert!(context.contains("id: operating-system"));
        assert!(context.contains("milestone_id: 01-bootloader"));

        let skill = read(&out.join(".gemini/skills/primer-build/SKILL.md"));
        assert!(skill.contains("name: primer-build"));
        assert!(skill.contains("description:"));
    }

    #[test]
    fn cursor_generation_creates_expected_files() {
        let source = RecipeSource::Bundled;
        let recipe = recipe::load_by_id(&source, "operating-system").expect("recipe should load");
        let out = temp_dir("cursor-gen");
        let recipe_path = recipe::materialize_into_workspace(&source, "operating-system", &out)
            .expect("recipe materialization should succeed");

        generate(
            &recipe,
            &recipe_path,
            &out,
            Tool::Cursor,
            "learner",
            "01-bootloader",
        )
        .expect("adapter generation should succeed");

        assert!(out.join("AGENTS.md").exists());
        assert!(out.join(".cursor/skills/primer-build/SKILL.md").exists());
        assert!(out.join(".cursor/skills/primer-track/SKILL.md").exists());
        assert!(out.join(".cursor/skills/primer-verify/SKILL.md").exists());

        let context = read(&out.join("AGENTS.md"));
        assert!(context.contains("kind: recipe"));
        assert!(context.contains("id: operating-system"));
        assert!(context.contains("milestone_id: 01-bootloader"));

        let skill = read(&out.join(".cursor/skills/primer-build/SKILL.md"));
        assert!(skill.contains("name: primer-build"));
        assert!(skill.contains("description:"));
    }
}
