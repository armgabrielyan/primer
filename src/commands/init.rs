use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use crate::cli::{InitArgs, Tool};
use crate::recipe;
use crate::ui;
use crate::workspace;

pub fn run(primer_root: &Path, args: InitArgs) -> Result<()> {
    let recipe = recipe::load_by_id(primer_root, &args.recipe_id)?;
    let milestone = recipe::resolve_initial_milestone(&recipe, args.milestone.as_deref())?;
    let workspace = workspace::prepare(primer_root, &args.path, args.force, args.dry_run)?;

    if args.dry_run {
        print_dry_run(&recipe, milestone.id.as_str(), &workspace.target_dir, &args);
        return Ok(());
    }

    validate_recipe(primer_root, &recipe.path)?;
    generate_adapter(
        primer_root,
        &recipe.path,
        &workspace.target_dir,
        args.tool,
        args.track.as_str(),
        milestone.id.as_str(),
    )?;

    print_success(
        &recipe,
        milestone.id.as_str(),
        milestone.title.as_str(),
        &workspace,
        &args,
    );
    Ok(())
}

fn validate_recipe(primer_root: &Path, recipe_dir: &Path) -> Result<()> {
    let spinner = ui::spinner("Validating recipe contract...");
    let output = Command::new(primer_root.join("scripts/validate-recipe"))
        .arg(recipe_dir)
        .output()
        .with_context(|| "failed to execute recipe validation".to_string())?;
    spinner.finish_and_clear();

    if !output.status.success() {
        bail!(
            "recipe validation failed:\n{}",
            render_command_output(&output)
        );
    }

    Ok(())
}

fn generate_adapter(
    primer_root: &Path,
    recipe_dir: &Path,
    output_dir: &Path,
    tool: Tool,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let spinner = ui::spinner("Generating Primer adapter files...");
    let script = match tool {
        Tool::Codex => "generate-codex-adapter",
        Tool::Claude => "generate-claude-adapter",
    };

    let output = Command::new(primer_root.join("scripts").join(script))
        .arg(recipe_dir)
        .arg("--output-dir")
        .arg(output_dir)
        .arg("--track")
        .arg(track)
        .arg("--milestone-id")
        .arg(milestone_id)
        .output()
        .with_context(|| "failed to execute adapter generator".to_string())?;
    spinner.finish_and_clear();

    if !output.status.success() {
        bail!(
            "adapter generation failed:\n{}",
            render_command_output(&output)
        );
    }

    Ok(())
}

fn render_command_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => format!("process exited with status {}", output.status),
    }
}

fn print_dry_run(
    recipe: &recipe::Recipe,
    milestone_id: &str,
    workspace_dir: &Path,
    args: &InitArgs,
) {
    ui::section("Primer init dry run");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Recipe".to_string(),
            value: recipe.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Title".to_string(),
            value: recipe.title.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Tool".to_string(),
            value: args.tool.display_name().to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: args.track.as_str().to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Milestone".to_string(),
            value: milestone_id.to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workspace".to_string(),
            value: workspace_dir.display().to_string(),
            value_color: Some(comfy_table::Color::Cyan),
        },
    ]);
    println!();
    ui::section("Planned actions");
    ui::numbered_steps(&[
        "Validate the recipe contract".to_string(),
        "Create the workspace directory if needed".to_string(),
        format!(
            "Generate {} adapter files into the workspace",
            args.tool.display_name()
        ),
    ]);
}

fn print_success(
    recipe: &recipe::Recipe,
    milestone_id: &str,
    milestone_title: &str,
    workspace: &workspace::PreparedWorkspace,
    args: &InitArgs,
) {
    ui::success("Initialized Primer workspace");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Recipe".to_string(),
            value: recipe.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Title".to_string(),
            value: recipe.title.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Stack".to_string(),
            value: recipe.stack_id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Tool".to_string(),
            value: args.tool.display_name().to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Track".to_string(),
            value: args.track.as_str().to_string(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workspace".to_string(),
            value: workspace.target_dir.display().to_string(),
            value_color: Some(comfy_table::Color::Cyan),
        },
        ui::KeyValueRow {
            key: "Current milestone".to_string(),
            value: format!("{milestone_id} ({milestone_title})"),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Workspace status".to_string(),
            value: if workspace.existed {
                "existing directory".to_string()
            } else {
                "new directory".to_string()
            },
            value_color: None,
        },
    ]);
    println!();
    ui::section("Next");
    let steps = match args.tool {
        Tool::Codex => {
            vec![
                format!(
                    "Open {} in Codex",
                    ui::code(workspace.target_dir.display().to_string())
                ),
                format!("Run the {}", ui::reference("skill", "primer-build")),
                format!(
                    "Use the {} when the milestone is complete",
                    ui::reference("skill", "primer-check")
                ),
            ]
        }
        Tool::Claude => {
            vec![
                format!(
                    "Open {} in Claude Code",
                    ui::code(workspace.target_dir.display().to_string())
                ),
                format!("Run the {}", ui::reference("skill", "primer-build")),
                format!(
                    "Use the {} when the milestone is complete",
                    ui::reference("skill", "primer-check")
                ),
            ]
        }
    };
    ui::numbered_steps(&steps);
}
