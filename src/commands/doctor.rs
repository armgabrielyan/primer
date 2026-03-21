use anyhow::{Result, bail};
use comfy_table::Color;
use std::collections::BTreeMap;
use std::path::Path;
use which::which;

use crate::cli::DoctorArgs;
use crate::recipe;
use crate::ui::{self, DoctorRow};

pub fn run(primer_root: &Path, args: DoctorArgs) -> Result<()> {
    let recipe = match args.recipe_id.as_deref() {
        Some(recipe_id) => recipe::load_by_id(primer_root, recipe_id)?,
        None => recipe::default_recipe(primer_root)?,
    };
    let milestone = recipe::resolve_initial_milestone(&recipe, args.milestone.as_deref())?;
    let milestone_index = recipe::milestone_index(&recipe, &milestone.id)?;

    let spinner = ui::spinner("Inspecting local toolchain...");
    let checks = build_checks(&recipe);
    let rows = checks
        .iter()
        .map(|check| doctor_row(check, milestone_index))
        .collect::<Vec<_>>();
    spinner.finish_and_clear();

    ui::section("Primer doctor");
    println!();
    ui::key_value_table(&[
        ui::KeyValueRow {
            key: "Recipe".to_string(),
            value: recipe.id.clone(),
            value_color: None,
        },
        ui::KeyValueRow {
            key: "Milestone".to_string(),
            value: format!("{} ({})", milestone.id, milestone.title),
            value_color: None,
        },
    ]);
    println!();
    ui::display_doctor_table(&rows);

    let missing_now = rows
        .iter()
        .filter(|row| row.status == "Missing" && row.when == "Required now")
        .count();

    println!();
    if missing_now == 0 {
        ui::success("All tools required for this milestone are available.");
        return Ok(());
    }

    bail!(
        "{} required tool(s) are missing for milestone {}",
        missing_now,
        milestone.id
    )
}

struct ToolCheck {
    tool: String,
    required_from: usize,
    first_required_by: String,
}

fn build_checks(recipe: &recipe::Recipe) -> Vec<ToolCheck> {
    let mut first_seen: BTreeMap<String, usize> = BTreeMap::new();

    for (index, milestone) in recipe.milestones.iter().enumerate() {
        for tool in &milestone.prerequisites {
            first_seen.entry(tool.clone()).or_insert(index);
        }
    }

    first_seen
        .into_iter()
        .map(|(tool, required_from)| ToolCheck {
            first_required_by: recipe.milestones[required_from].id.clone(),
            tool,
            required_from,
        })
        .collect()
}

fn doctor_row(check: &ToolCheck, milestone_index: usize) -> DoctorRow {
    let location = which(&check.tool)
        .ok()
        .map(|path| path.display().to_string());
    let found = location.is_some();
    let required_now = milestone_index >= check.required_from;

    let (status, status_color) = if found {
        ("Found".to_string(), Color::Green)
    } else {
        ("Missing".to_string(), Color::Red)
    };

    let (when, when_color) = if required_now {
        ("Required now".to_string(), Color::Yellow)
    } else {
        ("Later".to_string(), Color::Blue)
    };

    DoctorRow {
        tool: check.tool.clone(),
        status,
        status_color,
        when,
        when_color,
        location,
        notes: format!("First required by {}", check.first_required_by),
    }
}
