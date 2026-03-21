use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::recipe;
use crate::state;
use crate::ui;

pub fn run(primer_root: &Path, workspace_hint: &Path) -> Result<()> {
    let mut state = state::load_from_workspace(workspace_hint)?;
    let recipe = recipe::load_by_id(primer_root, &state.recipe_id)?;
    let milestone = recipe::resolve_initial_milestone(&recipe, Some(&state.milestone_id))?;

    if state.recipe_path != recipe.path {
        bail!(
            "workspace state points to {}, but resolved recipe is {}",
            state.recipe_path.display(),
            recipe.path.display()
        );
    }

    let check_script = recipe
        .path
        .join("milestones")
        .join(&milestone.id)
        .join("tests")
        .join("check.sh");

    if !check_script.is_file() {
        bail!(
            "milestone check script not found at {}",
            check_script.display()
        );
    }

    ui::section("Primer check");
    println!();
    ui::info(&format!(
        "Running {} for {} from {}",
        ui::code(check_script.display().to_string()),
        ui::code(&milestone.id),
        ui::code(state.workspace_root.display().to_string())
    ));
    println!();

    let status = Command::new("bash")
        .arg(&check_script)
        .current_dir(&state.workspace_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute {}", check_script.display()))?;

    if !status.success() {
        bail!(
            "milestone {} check failed; state was not updated",
            milestone.id
        );
    }

    state.verified_milestone_id = Some(milestone.id.clone());
    state::write(&state)?;

    println!();
    ui::success(&format!("Verified {}", milestone.id));
    println!(
        "The current milestone is now marked as verified in {}. You can use the {} next.",
        ui::code(state.context_path.display().to_string()),
        ui::reference("skill", "primer-next-milestone")
    );

    Ok(())
}
