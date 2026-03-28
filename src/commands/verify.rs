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
    let verify_command = resolve_verify_command(&checks_dir)?;

    ui::section("Primer verify");
    println!();
    ui::info(&format!(
        "Running {} for {} from {}",
        ui::code(verify_command.script.display().to_string()),
        ui::code(&milestone.id),
        ui::code(state.workspace_root.display().to_string())
    ));
    println!();

    let status = Command::new(&verify_command.program)
        .args(&verify_command.args)
        .current_dir(&state.workspace_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute {}", verify_command.script.display()))?;

    if !status.success() {
        if state.verified_milestone_id.as_deref() == Some(milestone.id.as_str()) {
            state.verified_milestone_id = None;
            state::write(&state)?;
            bail!(
                "milestone {} verification failed; current verified state was cleared",
                milestone.id
            );
        }

        bail!("milestone {} verification failed", milestone.id);
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

fn resolve_verify_command(checks_dir: &Path) -> Result<CheckCommand> {
    let verify_shell_script = checks_dir.join("verify.sh");
    let check_shell_script = checks_dir.join("check.sh");

    #[cfg(windows)]
    {
        let verify_cmd_script = checks_dir.join("verify.cmd");
        if verify_cmd_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("cmd.exe"),
                args: vec![
                    OsString::from("/D"),
                    OsString::from("/C"),
                    verify_cmd_script.as_os_str().to_os_string(),
                ],
                script: verify_cmd_script,
            });
        }

        let verify_powershell_script = checks_dir.join("verify.ps1");
        if verify_powershell_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("powershell"),
                args: vec![
                    OsString::from("-ExecutionPolicy"),
                    OsString::from("Bypass"),
                    OsString::from("-File"),
                    verify_powershell_script.as_os_str().to_os_string(),
                ],
                script: verify_powershell_script,
            });
        }

        if verify_shell_script.is_file() && which("bash").is_ok() {
            return Ok(CheckCommand {
                program: OsString::from("bash"),
                args: vec![verify_shell_script.as_os_str().to_os_string()],
                script: verify_shell_script,
            });
        }

        let check_cmd_script = checks_dir.join("check.cmd");
        if check_cmd_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("cmd.exe"),
                args: vec![
                    OsString::from("/D"),
                    OsString::from("/C"),
                    check_cmd_script.as_os_str().to_os_string(),
                ],
                script: check_cmd_script,
            });
        }

        let check_powershell_script = checks_dir.join("check.ps1");
        if check_powershell_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("powershell"),
                args: vec![
                    OsString::from("-ExecutionPolicy"),
                    OsString::from("Bypass"),
                    OsString::from("-File"),
                    check_powershell_script.as_os_str().to_os_string(),
                ],
                script: check_powershell_script,
            });
        }

        if check_shell_script.is_file() && which("bash").is_ok() {
            return Ok(CheckCommand {
                program: OsString::from("bash"),
                args: vec![check_shell_script.as_os_str().to_os_string()],
                script: check_shell_script,
            });
        }

        bail!(
            "milestone verification script not found for Windows in {}; expected verify.cmd, verify.ps1, verify.sh, check.cmd, check.ps1, or check.sh with bash on PATH",
            checks_dir.display()
        );
    }

    #[cfg(not(windows))]
    {
        if verify_shell_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("bash"),
                args: vec![verify_shell_script.as_os_str().to_os_string()],
                script: verify_shell_script,
            });
        }

        if check_shell_script.is_file() {
            return Ok(CheckCommand {
                program: OsString::from("bash"),
                args: vec![check_shell_script.as_os_str().to_os_string()],
                script: check_shell_script,
            });
        }

        bail!(
            "milestone verification script not found in {}; expected verify.sh or check.sh",
            checks_dir.display()
        );
    }
}
