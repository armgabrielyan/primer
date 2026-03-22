use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(windows)]
use which::which;

use crate::recipe;
use crate::state;
use crate::ui;

pub fn run(workspace_hint: &Path) -> Result<()> {
    let mut state = state::load_from_workspace(workspace_hint)?;
    let recipe = recipe::load_from_path(&state.recipe_path)?;
    let milestone = recipe::resolve_initial_milestone(&recipe, Some(&state.milestone_id))?;

    let checks_dir = recipe
        .path
        .join("milestones")
        .join(&milestone.id)
        .join("tests");
    let check_command = resolve_check_command(&checks_dir)?;

    ui::section("Primer check");
    println!();
    ui::info(&format!(
        "Running {} for {} from {}",
        ui::code(check_command.script.display().to_string()),
        ui::code(&milestone.id),
        ui::code(state.workspace_root.display().to_string())
    ));
    println!();

    let status = Command::new(&check_command.program)
        .args(&check_command.args)
        .current_dir(&state.workspace_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute {}", check_command.script.display()))?;

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

struct CheckCommand {
    program: OsString,
    args: Vec<OsString>,
    script: PathBuf,
}

fn resolve_check_command(checks_dir: &Path) -> Result<CheckCommand> {
    let shell_script = checks_dir.join("check.sh");

    #[cfg(windows)]
    {
        let cmd_script = checks_dir.join("check.cmd");
        if cmd_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("cmd"),
                args: vec![OsString::from("/C"), cmd_script.as_os_str().to_os_string()],
                script: cmd_script,
            });
        }

        let powershell_script = checks_dir.join("check.ps1");
        if powershell_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("powershell"),
                args: vec![
                    OsString::from("-ExecutionPolicy"),
                    OsString::from("Bypass"),
                    OsString::from("-File"),
                    powershell_script.as_os_str().to_os_string(),
                ],
                script: powershell_script,
            });
        }

        if shell_script.is_file() && which("bash").is_ok() {
            return Ok(CheckCommand {
                program: OsString::from("bash"),
                args: vec![shell_script.as_os_str().to_os_string()],
                script: shell_script,
            });
        }

        bail!(
            "milestone check script not found for Windows in {}; expected check.cmd, check.ps1, or check.sh with bash on PATH",
            checks_dir.display()
        );
    }

    #[cfg(not(windows))]
    {
        if shell_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("bash"),
                args: vec![shell_script.as_os_str().to_os_string()],
                script: shell_script,
            });
        }

        bail!(
            "milestone check script not found at {}",
            shell_script.display()
        );
    }
}
