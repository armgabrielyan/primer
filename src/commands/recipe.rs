use anyhow::{Context, Result, bail};
use console::style;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::RecipeLintArgs;
use crate::paths;
use crate::recipe as recipe_catalog;
use crate::ui;
use crate::validation::{self, LintFinding, LintSeverity, RecipeLintReport};

#[derive(Debug, Serialize)]
struct RecipeLintResponse {
    checked: usize,
    clean: bool,
    errors: usize,
    warnings: usize,
    advice: usize,
    recipes: Vec<RecipeLintReport>,
}

pub fn lint(
    source: &recipe_catalog::RecipeSource,
    workspace_hint: &Path,
    args: RecipeLintArgs,
) -> Result<()> {
    let targets = resolve_targets(source, workspace_hint, &args)?;
    let reports = targets
        .iter()
        .map(|target| validation::lint_recipe(target))
        .collect::<Vec<_>>();
    let response = RecipeLintResponse {
        checked: reports.len(),
        clean: reports.iter().all(RecipeLintReport::is_clean),
        errors: reports
            .iter()
            .map(|report| report.count_by_severity(LintSeverity::Error))
            .sum(),
        warnings: reports
            .iter()
            .map(|report| report.count_by_severity(LintSeverity::Warning))
            .sum(),
        advice: reports
            .iter()
            .map(|report| report.count_by_severity(LintSeverity::Advice))
            .sum(),
        recipes: reports,
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&response)
                .context("failed to serialize recipe lint output")?
        );
    } else {
        print_human_report(&response);
    }

    if response
        .recipes
        .iter()
        .any(RecipeLintReport::has_blocking_findings)
    {
        bail!(
            "recipe lint found {} error(s) and {} warning(s)",
            response.errors,
            response.warnings
        );
    }

    Ok(())
}

fn resolve_targets(
    source: &recipe_catalog::RecipeSource,
    workspace_hint: &Path,
    args: &RecipeLintArgs,
) -> Result<Vec<PathBuf>> {
    if let Some(path) = args.path.as_deref() {
        let resolved = paths::absolute(path)?;
        ensure_recipe_dir(&resolved)?;
        return Ok(vec![resolved]);
    }

    if workspace_hint.join("recipe.yaml").is_file() {
        if let Some(requested_id) = args.recipe_id.as_deref()
            && let Ok(recipe) = recipe_catalog::load_from_path(workspace_hint)
            && recipe.id != requested_id
        {
            bail!(
                "current directory recipe is '{}' not '{}'",
                recipe.id,
                requested_id
            );
        }

        return Ok(vec![paths::canonicalize(workspace_hint)?]);
    }

    if workspace_hint.join("recipes").is_dir() {
        return collect_recipe_dirs(&workspace_hint.join("recipes"), args.recipe_id.as_deref());
    }

    if let recipe_catalog::RecipeSource::Filesystem(primer_root) = source {
        return collect_recipe_dirs(&primer_root.join("recipes"), args.recipe_id.as_deref());
    }

    bail!(
        "recipe lint needs filesystem recipe files; run it from a recipe directory, a repository checkout with recipes/, or pass --path"
    )
}

fn collect_recipe_dirs(recipes_root: &Path, requested_id: Option<&str>) -> Result<Vec<PathBuf>> {
    if let Some(requested_id) = requested_id {
        let recipe_dir = recipes_root.join(requested_id);
        ensure_recipe_dir(&recipe_dir)?;
        return Ok(vec![paths::canonicalize(&recipe_dir)?]);
    }

    let mut recipes = fs::read_dir(recipes_root)
        .with_context(|| format!("failed to read {}", recipes_root.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir() && path.join("recipe.yaml").is_file())
        .collect::<Vec<_>>();
    recipes.sort();

    if recipes.is_empty() {
        bail!("no recipe directories found in {}", recipes_root.display());
    }

    recipes
        .into_iter()
        .map(|path| paths::canonicalize(&path))
        .collect()
}

fn ensure_recipe_dir(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("recipe directory not found at {}", path.display());
    }
    if !path.join("recipe.yaml").is_file() {
        bail!("recipe.yaml not found in {}", path.display());
    }
    Ok(())
}

fn print_human_report(response: &RecipeLintResponse) {
    ui::section("Primer recipe lint");

    for report in &response.recipes {
        println!();
        println!(
            "{} {}",
            style("recipe").bold(),
            style(report.recipe_id.as_deref().unwrap_or("unknown"))
                .cyan()
                .bold()
        );
        println!("  path: {}", report.path.display());
        println!("  milestones: {}", report.milestone_count);

        if report.findings.is_empty() {
            println!("  status: {}", style("clean").green().bold());
            continue;
        }

        println!("  findings:");
        for finding in &report.findings {
            print_finding(finding);
        }
    }

    println!();
    ui::section("Summary");
    println!(
        "{} recipe(s) checked, {} error(s), {} warning(s), {} advice item(s).",
        response.checked, response.errors, response.warnings, response.advice
    );

    if response.errors == 0 && response.warnings == 0 {
        ui::success("Recipe lint passed.");
    }
}

fn print_finding(finding: &LintFinding) {
    let severity = match finding.severity {
        LintSeverity::Error => style(finding.severity.label()).red().bold(),
        LintSeverity::Warning => style(finding.severity.label()).yellow().bold(),
        LintSeverity::Advice => style(finding.severity.label()).blue().bold(),
    };

    if let Some(milestone_id) = finding.milestone_id.as_deref() {
        println!(
            "    - {} [{}] {}: {}",
            severity,
            finding.code,
            style(milestone_id).cyan(),
            finding.message
        );
        return;
    }

    println!("    - {} [{}] {}", severity, finding.code, finding.message);
}
