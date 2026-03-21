use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RecipeSummary {
    pub id: String,
    pub title: String,
    pub difficulty: String,
    pub path: PathBuf,
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
}

pub fn discover(primer_root: &Path) -> Result<Vec<RecipeSummary>> {
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
            path: recipe.path,
        });
    }

    recipes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(recipes)
}

pub fn load_by_id(primer_root: &Path, recipe_id: &str) -> Result<Recipe> {
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

pub fn default_recipe(primer_root: &Path) -> Result<Recipe> {
    let recipes = discover(primer_root)?;
    match recipes.as_slice() {
        [] => bail!(
            "no recipes found in {}",
            primer_root.join("recipes").display()
        ),
        [only] => load_by_id(primer_root, &only.id),
        _ => bail!("multiple recipes available; pass an explicit recipe id"),
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
    let parsed: RecipeDoc = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse {}", recipe_yaml.display()))?;

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
            })
            .collect(),
    })
}
