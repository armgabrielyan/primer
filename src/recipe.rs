use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bundled;
use crate::paths;

#[derive(Debug, Clone)]
pub struct RecipeSummary {
    pub id: String,
    pub title: String,
    pub difficulty: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub difficulty: String,
    pub stack_id: String,
    pub path: PathBuf,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub prerequisites: Vec<String>,
    pub goal: Option<String>,
    pub verification_summary: Option<String>,
    pub expected_artifacts: Vec<String>,
    pub estimated_verify_minutes: Option<u32>,
    pub split_if_stuck: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RecipeSource {
    Bundled,
    Filesystem(PathBuf),
}

#[derive(Debug, Deserialize)]
struct RecipeDoc {
    id: String,
    title: String,
    difficulty: String,
    stack: StackDoc,
    milestones: Vec<MilestoneDoc>,
}

#[derive(Debug, Deserialize)]
struct StackDoc {
    id: String,
}

#[derive(Debug, Deserialize)]
struct MilestoneDoc {
    id: String,
    title: String,
    prerequisites: Vec<String>,
    goal: Option<String>,
    verification_summary: Option<String>,
    #[serde(default)]
    expected_artifacts: Vec<String>,
    estimated_verify_minutes: Option<u32>,
    split_if_stuck: Option<String>,
}

pub fn source(primer_root: Option<&Path>) -> Result<RecipeSource> {
    match primer_root {
        Some(path) => {
            let recipes_dir = path.join("recipes");
            if !recipes_dir.is_dir() {
                bail!(
                    "Primer recipes directory not found at {}",
                    recipes_dir.display()
                );
            }
            Ok(RecipeSource::Filesystem(paths::canonicalize(path)?))
        }
        None => Ok(RecipeSource::Bundled),
    }
}

pub fn discover(source: &RecipeSource) -> Result<Vec<RecipeSummary>> {
    match source {
        RecipeSource::Bundled => bundled::recipes()
            .iter()
            .map(|bundled| {
                let recipe = bundled_recipe_to_recipe(bundled)?;
                Ok(RecipeSummary {
                    id: recipe.id,
                    title: recipe.title,
                    difficulty: recipe.difficulty,
                    location: bundled.location.to_string(),
                })
            })
            .collect(),
        RecipeSource::Filesystem(primer_root) => {
            let recipes_dir = recipes_dir(primer_root)?;
            let mut recipes = Vec::new();

            for entry in fs::read_dir(&recipes_dir)
                .with_context(|| format!("failed to read {}", recipes_dir.display()))?
            {
                let entry = entry.with_context(|| "failed to read directory entry".to_string())?;
                let path = entry.path();
                if !path.is_dir() || !path.join("recipe.yaml").is_file() {
                    continue;
                }

                let recipe = load_from_dir(&path)?;
                recipes.push(RecipeSummary {
                    id: recipe.id,
                    title: recipe.title,
                    difficulty: recipe.difficulty,
                    location: path.display().to_string(),
                });
            }

            recipes.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(recipes)
        }
    }
}

pub fn load_by_id(source: &RecipeSource, recipe_id: &str) -> Result<Recipe> {
    match source {
        RecipeSource::Bundled => {
            let bundled = bundled::require_recipe(recipe_id)?;
            bundled_recipe_to_recipe(bundled)
        }
        RecipeSource::Filesystem(primer_root) => {
            let recipe_dir = recipes_dir(primer_root)?.join(recipe_id);
            if !recipe_dir.is_dir() {
                bail!(
                    "recipe '{}' not found in {}",
                    recipe_id,
                    primer_root.join("recipes").display()
                );
            }
            let recipe = load_from_dir(&recipe_dir)?;
            if recipe.id != recipe_id {
                bail!(
                    "recipe id mismatch: requested '{}', recipe.yaml declares '{}'",
                    recipe_id,
                    recipe.id
                );
            }
            Ok(recipe)
        }
    }
}

pub fn load_from_path(recipe_dir: &Path) -> Result<Recipe> {
    load_from_dir(recipe_dir)
}

pub fn default_recipe(source: &RecipeSource) -> Result<Recipe> {
    let recipes = discover(source)?;
    match recipes.as_slice() {
        [] => bail!("no recipes found"),
        [only] => load_by_id(source, &only.id),
        _ => bail!("multiple recipes available; pass an explicit recipe id"),
    }
}

pub fn materialize_into_workspace(
    source: &RecipeSource,
    recipe_id: &str,
    workspace_root: &Path,
) -> Result<PathBuf> {
    let recipe_dir = workspace_root
        .join(".primer")
        .join("recipes")
        .join(recipe_id);
    if recipe_dir.exists() {
        fs::remove_dir_all(&recipe_dir)
            .with_context(|| format!("failed to clear {}", recipe_dir.display()))?;
    }

    match source {
        RecipeSource::Bundled => bundled::materialize_recipe(recipe_id, &recipe_dir),
        RecipeSource::Filesystem(primer_root) => {
            let source_dir = recipes_dir(primer_root)?.join(recipe_id);
            copy_dir(&source_dir, &recipe_dir)?;
            Ok(recipe_dir)
        }
    }
}

pub fn resolve_initial_milestone<'a>(
    recipe: &'a Recipe,
    requested: Option<&str>,
) -> Result<&'a Milestone> {
    if let Some(requested) = requested {
        return recipe
            .milestones
            .iter()
            .find(|m| m.id == requested)
            .ok_or_else(|| {
                anyhow!(
                    "milestone '{}' is not declared for recipe '{}'",
                    requested,
                    recipe.id
                )
            });
    }

    recipe
        .milestones
        .first()
        .ok_or_else(|| anyhow!("recipe '{}' declares no milestones", recipe.id))
}

pub fn milestone_index(recipe: &Recipe, milestone_id: &str) -> Result<usize> {
    recipe
        .milestones
        .iter()
        .position(|m| m.id == milestone_id)
        .ok_or_else(|| {
            anyhow!(
                "milestone '{}' is not declared for recipe '{}'",
                milestone_id,
                recipe.id
            )
        })
}

fn bundled_recipe_to_recipe(bundled_recipe: &bundled::BundledRecipe) -> Result<Recipe> {
    let raw = bundled_recipe
        .assets
        .iter()
        .find(|asset| asset.relative_path == "recipe.yaml")
        .map(|asset| asset.contents)
        .ok_or_else(|| {
            anyhow!(
                "bundled recipe '{}' is missing recipe.yaml",
                bundled_recipe.id
            )
        })?;
    parse_recipe_doc(raw, Path::new(bundled_recipe.location))
}

fn recipes_dir(primer_root: &Path) -> Result<PathBuf> {
    let recipes_dir = primer_root.join("recipes");
    if !recipes_dir.is_dir() {
        bail!(
            "Primer recipes directory not found at {}",
            recipes_dir.display()
        );
    }
    Ok(recipes_dir)
}

fn load_from_dir(path: &Path) -> Result<Recipe> {
    let recipe_yaml = path.join("recipe.yaml");
    let raw = fs::read_to_string(&recipe_yaml)
        .with_context(|| format!("failed to read {}", recipe_yaml.display()))?;
    parse_recipe_doc(&raw, path)
}

fn parse_recipe_doc(raw: &str, path: &Path) -> Result<Recipe> {
    let parsed: RecipeDoc = serde_yaml::from_str(raw)
        .with_context(|| format!("failed to parse {}", path.join("recipe.yaml").display()))?;

    Ok(Recipe {
        id: parsed.id,
        title: parsed.title,
        difficulty: parsed.difficulty,
        stack_id: parsed.stack.id,
        path: path.to_path_buf(),
        milestones: parsed
            .milestones
            .into_iter()
            .map(|m| Milestone {
                id: m.id,
                title: m.title,
                prerequisites: m.prerequisites,
                goal: m.goal,
                verification_summary: m.verification_summary,
                expected_artifacts: m.expected_artifacts,
                estimated_verify_minutes: m.estimated_verify_minutes,
                split_if_stuck: m.split_if_stuck,
            })
            .collect(),
    })
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry.with_context(|| "failed to read directory entry".to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = entry
            .metadata()
            .with_context(|| format!("failed to read metadata for {}", source_path.display()))?;
        if metadata.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}
