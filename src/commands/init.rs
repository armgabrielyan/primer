use anyhow::{Result, bail};
use std::path::Path;

use crate::adapter;
use crate::cli::{InitArgs, Tool};
use crate::recipe;
use crate::ui;
use crate::validation;
use crate::workspace;

pub fn run(source: &recipe::RecipeSource, args: InitArgs) -> Result<()> {
    let recipe = recipe::load_by_id(source, &args.recipe_id)?;
    let milestone = recipe::resolve_initial_milestone(&recipe, args.milestone.as_deref())?;
    let workspace = workspace::prepare(&args.path, args.force, args.dry_run)?;

    if args.dry_run {
        print_dry_run(&recipe, milestone.id.as_str(), &workspace.target_dir, &args);
        return Ok(());
    }

    let recipe_snapshot =
        recipe::materialize_into_workspace(source, &recipe.id, &workspace.target_dir)?;
    validate_recipe(&recipe_snapshot)?;
    generate_adapter(
        &recipe_snapshot,
        &workspace.target_dir,
        &recipe,
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

fn validate_recipe(recipe_dir: &Path) -> Result<()> {
    let spinner = ui::spinner("Validating recipe contract...");
    let result = validation::validate_recipe(recipe_dir);
    spinner.finish_and_clear();

    if let Err(err) = result {
        bail!("recipe validation failed:\n{err}");
    }

    Ok(())
}

fn generate_adapter(
    recipe_dir: &Path,
    output_dir: &Path,
    recipe: &recipe::Recipe,
    tool: Tool,
    track: &str,
    milestone_id: &str,
) -> Result<()> {
    let spinner = ui::spinner("Generating Primer adapter files...");
    let result = adapter::generate(recipe, recipe_dir, output_dir, tool, track, milestone_id);
    spinner.finish_and_clear();

    if let Err(err) = result {
        bail!("adapter generation failed:\n{err}");
    }

    Ok(())
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
        Tool::Opencode => {
            vec![
                format!(
                    "Open {} in OpenCode",
                    ui::code(workspace.target_dir.display().to_string())
                ),
                format!("Run the {}", ui::reference("skill", "primer-build")),
                format!(
                    "Use the {} when the milestone is complete",
                    ui::reference("skill", "primer-check")
                ),
            ]
        }
        Tool::Gemini => {
            vec![
                format!(
                    "Open {} in Gemini CLI",
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
