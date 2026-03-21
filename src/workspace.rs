use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

pub struct PreparedWorkspace {
    pub target_dir: PathBuf,
    pub existed: bool,
}

pub fn resolve_primer_root(primer_root_hint: &Path) -> Result<PathBuf> {
    if primer_root_hint.join("recipes").is_dir() {
        return fs::canonicalize(primer_root_hint)
            .with_context(|| format!("failed to resolve {}", primer_root_hint.display()));
    }

    if let Some(resolved) = resolve_from_workspace_state(primer_root_hint)? {
        return Ok(resolved);
    }

    bail!(
        "Primer recipes directory not found at {}",
        primer_root_hint.join("recipes").display()
    )
}

pub fn prepare(
    primer_root: &Path,
    requested_path: &Path,
    force: bool,
    dry_run: bool,
) -> Result<PreparedWorkspace> {
    let primer_root = fs::canonicalize(primer_root)
        .with_context(|| format!("failed to resolve {}", primer_root.display()))?;
    let target_dir = absolute_path(requested_path)?;

    if is_within(&target_dir, &primer_root) {
        bail!(
            "workspace path {} must be outside the Primer repository {}",
            target_dir.display(),
            primer_root.display()
        );
    }

    let existed = target_dir.exists();
    if existed {
        if !target_dir.is_dir() {
            bail!("workspace path {} is not a directory", target_dir.display());
        }

        let mut entries = fs::read_dir(&target_dir)
            .with_context(|| format!("failed to read {}", target_dir.display()))?;
        if entries.next().is_some() && !force {
            bail!(
                "workspace directory {} is not empty; use --force to initialize anyway",
                target_dir.display()
            );
        }
    } else if !dry_run {
        fs::create_dir_all(&target_dir)
            .with_context(|| format!("failed to create {}", target_dir.display()))?;
    }

    Ok(PreparedWorkspace {
        target_dir,
        existed,
    })
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path)
            .with_context(|| format!("failed to resolve {}", path.display()));
    }

    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    Ok(current_dir.join(path))
}

fn is_within(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

fn resolve_from_workspace_state(dir: &Path) -> Result<Option<PathBuf>> {
    let dir = absolute_path(dir)?;

    for filename in ["CLAUDE.md", "AGENTS.md"] {
        let path = dir.join(filename);
        if !path.is_file() {
            continue;
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if let Some(recipe_path) = extract_yaml_value(&text, "recipe_path") {
            let recipe_path = PathBuf::from(recipe_path);
            if let Some(primer_root) = recipe_path.parent().and_then(Path::parent) {
                let primer_root = fs::canonicalize(primer_root)
                    .with_context(|| format!("failed to resolve {}", primer_root.display()))?;
                if primer_root.join("recipes").is_dir() {
                    return Ok(Some(primer_root));
                }
            }
        }
    }

    Ok(None)
}

fn extract_yaml_value(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    text.lines().map(str::trim).find_map(|line| {
        line.strip_prefix(&prefix)
            .map(|value| value.trim().to_string())
    })
}
