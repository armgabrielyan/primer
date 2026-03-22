use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

use crate::bundled;
use crate::cli::Tool;
use crate::recipe::Recipe;

const CODEX_SKILL_FILES: &[&str] = &[
    "build.md",
    "check.md",
    "explain.md",
    "status.md",
    "next-milestone.md",
];
const CLAUDE_COMMAND_FILES: &[&str] = &[
    "next-milestone.md",
    "check.md",
    "explain.md",
    "status.md",
    "build.md",
];

pub fn generate(
    recipe: &Recipe,
    recipe_path: &Path,
    output_dir: &Path,
    tool: Tool,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    ensure_valid_initial_milestone(recipe, milestone_id)?;
    match tool {
        Tool::Codex => generate_codex(recipe, recipe_path, output_dir, track, milestone_id),
        Tool::Claude => generate_claude(recipe, recipe_path, output_dir, track, milestone_id),
        Tool::Gemini => generate_gemini(recipe, recipe_path, output_dir, track, milestone_id),
        Tool::Opencode => generate_opencode(recipe, recipe_path, output_dir, track, milestone_id),
    }
}

fn ensure_valid_initial_milestone(recipe: &Recipe, milestone_id: &str) -> Result<()> {
    if recipe
        .milestones
        .iter()
        .any(|milestone| milestone.id == milestone_id)
    {
        return Ok(());
    }
    bail!(
        "{}: milestone_id '{}' is not declared in milestones",
        recipe.path.join("recipe.yaml").display(),
        milestone_id
    )
}

fn generate_codex(
    recipe: &Recipe,
    recipe_path: &Path,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let skills_root = output_dir.join(".agents").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &agents_md,
        render_adapter_context(recipe, recipe_path, output_dir, track, milestone_id),
    )?;

    for filename in CODEX_SKILL_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        let agents_dir = skill_dir.join("agents");
        fs::create_dir_all(&agents_dir)?;

        let body = match command_name {
            "check" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_skill_md(
                &skill_name,
                &format!(
                    "Use when the user wants to {} for the current Primer recipe in this repo workspace.",
                    command_name.replace('-', " ")
                ),
                &body,
            ),
        )?;
        fs::write(
            agents_dir.join("openai.yaml"),
            render_openai_yaml(
                &format!(
                    "{}: {}",
                    recipe.title,
                    title_case(&command_name.replace('-', " "))
                ),
                &format!(
                    "{} for the current Primer recipe",
                    capitalize(&command_name.replace('-', " "))
                ),
                &format!("Use $primer-{command_name} for the current recipe milestone."),
            ),
        )?;
    }

    Ok(())
}

fn generate_claude(
    recipe: &Recipe,
    recipe_path: &Path,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let claude_md = output_dir.join("CLAUDE.md");
    let commands_dir = output_dir.join(".claude").join("commands");
    fs::create_dir_all(&commands_dir)?;

    fs::write(
        &claude_md,
        render_adapter_context(recipe, recipe_path, output_dir, track, milestone_id),
    )?;

    for filename in CLAUDE_COMMAND_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let target_name = format!("primer-{command_name}.md");
        let contents = match command_name {
            "check" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_claude_command(command_name, shared_body)
            }
            "build" => render_build_claude_command(shared_body),
            _ => shared_body.to_string(),
        };
        fs::write(commands_dir.join(target_name), contents)?;
    }

    Ok(())
}

fn generate_opencode(
    recipe: &Recipe,
    recipe_path: &Path,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let skills_root = output_dir.join(".opencode").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &agents_md,
        render_adapter_context(recipe, recipe_path, output_dir, track, milestone_id),
    )?;

    for filename in CODEX_SKILL_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        fs::create_dir_all(&skill_dir)?;

        let body = match command_name {
            "check" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_opencode_skill_md(
                &skill_name,
                &format!(
                    "Use when the user wants to {} for the current Primer recipe in this repo workspace.",
                    command_name.replace('-', " ")
                ),
                &body,
            ),
        )?;
    }

    Ok(())
}

fn generate_gemini(
    recipe: &Recipe,
    recipe_path: &Path,
    output_dir: &Path,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let gemini_md = output_dir.join("GEMINI.md");
    let skills_root = output_dir.join(".gemini").join("skills");
    fs::create_dir_all(&skills_root)?;

    fs::write(
        &gemini_md,
        render_adapter_context(recipe, recipe_path, output_dir, track, milestone_id),
    )?;

    for filename in CODEX_SKILL_FILES {
        let shared_body = bundled::require_shared_contract(filename)?;
        let command_name = filename.trim_end_matches(".md");
        let skill_name = format!("primer-{command_name}");
        let skill_dir = skills_root.join(&skill_name);
        fs::create_dir_all(&skill_dir)?;

        let body = match command_name {
            "check" | "explain" | "status" | "next-milestone" => {
                render_cli_backed_skill_body(command_name, shared_body)
            }
            "build" => render_build_skill_body(shared_body),
            _ => shared_body.to_string(),
        };

        fs::write(
            skill_dir.join("SKILL.md"),
            render_skill_md(
                &skill_name,
                &format!(
                    "Use when the user wants to {} for the current Primer recipe in this repo workspace.",
                    command_name.replace('-', " ")
                ),
                &body,
            ),
        )?;
    }

    Ok(())
}

fn render_adapter_context(
    recipe: &Recipe,
    recipe_path: &Path,
    workspace_root: &Path,
    track: &str,
    milestone_id: &str,
) -> String {
    format!(
        "# Primer — {}\n\n```yaml\nprimer_state:\n  recipe_id: {}\n  recipe_path: {}\n  workspace_root: {}\n  milestone_id: {}\n  verified_milestone_id: null\n  track: {}\n  stack_id: {}\n```\n\n## Recipe location\n\n{}/\n\n## Workspace root\n\n{}/\n\n## Rules\n\n- Always read the current milestone `agent.md` before starting work.\n- Work in this project workspace, not in the `primer` repository.\n- Build the current milestone in small steps and do not implement future milestones early.\n- Run current milestone `tests/check.sh` before declaring completion.\n- Only run `primer-next-milestone` after `primer-check` has marked the current milestone as verified.\n- Use the local `primer` CLI as the source of truth for `primer-check`, `primer-status`, `primer-explain`, and `primer-next-milestone`.\n- Use the generated Primer workflow actions for behavior rules.\n\n## Available workflow actions\n\n- `primer-build` — implement the current milestone step by step\n- `primer-next-milestone` — advance state only after the milestone is already verified\n- `primer-check` — run current milestone checks\n- `primer-explain` — show current milestone explanation\n- `primer-status` — show current state and progress\n",
        recipe.title,
        recipe.id,
        recipe_path.display(),
        workspace_root.display(),
        milestone_id,
        track,
        recipe.stack_id,
        recipe_path.display(),
        workspace_root.display()
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
        "Use the local Primer CLI as the source of truth for this workflow action.\n\n## Required action\n\n1. Run `primer {command_name}` from the current workspace root.\n2. Return the CLI output to the user faithfully.\n3. Do not manually edit `primer_state`; the CLI owns state transitions for this skill.\n4. If the `primer` executable is unavailable, stop and tell the user to install or build the Primer CLI.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_build_skill_body(shared_body: &str) -> String {
    format!(
        "Use the local Primer CLI to load the current milestone spec and track guidance before making changes.\n\n## Required action\n\n1. Run `primer build` from the current workspace root.\n2. Use that output as the current milestone contract and active track guidance.\n3. Then implement only the current milestone scope in the workspace.\n4. Recommend running `primer-check` when the milestone is complete.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_cli_backed_claude_command(command_name: &str, shared_body: &str) -> String {
    format!(
        "# Primer Skill: `primer-{command_name}`\n\nUse the local Primer CLI as the source of truth for this workflow action.\n\n## Required action\n\n1. Run `primer {command_name}` from the current workspace root.\n2. Return the CLI output to the user faithfully.\n3. Do not manually edit `primer_state`; the CLI owns state transitions for this skill.\n4. If the `primer` executable is unavailable, stop and tell the user to install or build the Primer CLI.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
}

fn render_build_claude_command(shared_body: &str) -> String {
    format!(
        "# Primer Skill: `primer-build`\n\nUse the local Primer CLI to load the current milestone spec and track guidance before making changes.\n\n## Required action\n\n1. Run `primer build` from the current workspace root.\n2. Use that output as the current milestone contract and active track guidance.\n3. Then implement only the current milestone scope in the workspace.\n4. Recommend running `primer-check` when the milestone is complete.\n\n## Shared contract reference\n\n{shared_body}\n"
    )
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
        assert!(out.join(".agents/skills/primer-check/SKILL.md").exists());

        let context = read(&out.join("AGENTS.md"));
        assert!(context.contains("recipe_id: operating-system"));
        assert!(context.contains("milestone_id: 01-bootloader"));
        assert!(context.contains(&format!("recipe_path: {}", recipe_path.display())));
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
        assert!(
            out.join(".claude/commands/primer-next-milestone.md")
                .exists()
        );

        let context = read(&out.join("CLAUDE.md"));
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
        assert!(out.join(".opencode/skills/primer-check/SKILL.md").exists());

        let context = read(&out.join("AGENTS.md"));
        assert!(context.contains("recipe_id: operating-system"));
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
        assert!(out.join(".gemini/skills/primer-check/SKILL.md").exists());

        let context = read(&out.join("GEMINI.md"));
        assert!(context.contains("recipe_id: operating-system"));
        assert!(context.contains("milestone_id: 01-bootloader"));

        let skill = read(&out.join(".gemini/skills/primer-build/SKILL.md"));
        assert!(skill.contains("name: primer-build"));
        assert!(skill.contains("description:"));
    }
}
