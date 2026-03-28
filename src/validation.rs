use anyhow::{Result, bail};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DIFFICULTY_VALUES: &[&str] = &["beginner", "intermediate", "advanced"];
const REQUIRED_MILESTONE_FILES: &[&str] = &[
    "spec.md",
    "agent.md",
    "explanation.md",
    "tests/check.sh",
    "demo.sh",
];

pub fn validate_recipe(recipe_target: &Path) -> Result<()> {
    let recipe_yaml = recipe_yaml_path(recipe_target);
    let recipe_dir = recipe_dir_path(recipe_target);
    let mut errors = validate_recipe_yaml(&recipe_yaml);
    errors.extend(validate_milestones(&recipe_dir));

    if errors.is_empty() {
        return Ok(());
    }

    bail!(errors.join("\n"))
}

pub fn validate_recipe_yaml(recipe_target: &Path) -> Vec<String> {
    let recipe_yaml = recipe_yaml_path(recipe_target);
    let mut errors = Vec::new();

    let raw = match fs::read_to_string(&recipe_yaml) {
        Ok(raw) => raw,
        Err(_) => {
            errors.push(error(&recipe_yaml, "recipe.yaml", "file not found"));
            return errors;
        }
    };

    let doc: Value = match serde_yaml::from_str(&raw) {
        Ok(doc) => doc,
        Err(err) => {
            errors.push(error(
                &recipe_yaml,
                "recipe.yaml",
                &format!("invalid YAML: {err}"),
            ));
            return errors;
        }
    };

    let Some(root) = doc.as_mapping() else {
        errors.push(error(&recipe_yaml, "<root>", "expected mapping/object"));
        return errors;
    };

    for key in [
        "id",
        "title",
        "description",
        "difficulty",
        "stack",
        "tracks",
        "milestones",
    ] {
        if get(root, key).is_none() {
            errors.push(error(&recipe_yaml, key, "is required"));
        }
    }

    match get_string(root, "id") {
        Some(id) => {
            if !is_kebab_case(id) {
                errors.push(error(
                    &recipe_yaml,
                    "id",
                    "must be kebab-case (e.g. operating-system)",
                ));
            }
        }
        None if get(root, "id").is_some() => {
            errors.push(error(&recipe_yaml, "id", "must be a string"));
        }
        None => {}
    }

    if let Some(difficulty) = get_string(root, "difficulty")
        && !DIFFICULTY_VALUES.contains(&difficulty)
    {
        errors.push(error(
            &recipe_yaml,
            "difficulty",
            &format!("must be one of {:?}", DIFFICULTY_VALUES),
        ));
    }

    if let Some(value) = get(root, "stack") {
        let Some(stack) = value.as_mapping() else {
            errors.push(error(&recipe_yaml, "stack", "must be an object"));
            return errors;
        };

        for key in ["id", "label", "tools"] {
            if get(stack, key).is_none() {
                errors.push(error(&recipe_yaml, &format!("stack.{key}"), "is required"));
            }
        }

        if get(stack, "id").is_some() && get_string(stack, "id").is_none() {
            errors.push(error(&recipe_yaml, "stack.id", "must be a string"));
        }
        if get(stack, "label").is_some() && get_string(stack, "label").is_none() {
            errors.push(error(&recipe_yaml, "stack.label", "must be a string"));
        }
        if let Some(tools) = get(stack, "tools") {
            validate_string_array(&mut errors, &recipe_yaml, "stack.tools", tools, true);
        }
    }

    if get(root, "stacks").is_some() {
        errors.push(error(
            &recipe_yaml,
            "stacks",
            "is not supported; use singular 'stack'",
        ));
    }

    if let Some(value) = get(root, "tracks") {
        let Some(tracks) = value.as_mapping() else {
            errors.push(error(&recipe_yaml, "tracks", "must be an object"));
            return errors;
        };

        for key in ["learner", "builder"] {
            let field = format!("tracks.{key}");
            let Some(track) = get(tracks, key) else {
                errors.push(error(&recipe_yaml, &field, "is required"));
                continue;
            };
            let Some(track) = track.as_mapping() else {
                errors.push(error(&recipe_yaml, &field, "must be an object"));
                continue;
            };
            match get_string(track, "description") {
                Some(_) => {}
                None => errors.push(error(
                    &recipe_yaml,
                    &format!("{field}.description"),
                    "must be a string",
                )),
            }
        }
    }

    if let Some(value) = get(root, "milestones") {
        let Some(milestones) = value.as_sequence() else {
            errors.push(error(
                &recipe_yaml,
                "milestones",
                "must be a non-empty array",
            ));
            return errors;
        };

        if milestones.is_empty() {
            errors.push(error(
                &recipe_yaml,
                "milestones",
                "must be a non-empty array",
            ));
            return errors;
        }

        let mut ids = Vec::new();
        for (index, item) in milestones.iter().enumerate() {
            let field_base = format!("milestones[{index}]");
            let Some(item) = item.as_mapping() else {
                errors.push(error(&recipe_yaml, &field_base, "must be an object"));
                continue;
            };

            for key in ["id", "title", "demo", "prerequisites"] {
                if get(item, key).is_none() {
                    errors.push(error(
                        &recipe_yaml,
                        &format!("{field_base}.{key}"),
                        "is required",
                    ));
                }
            }

            match get_string(item, "id") {
                Some(id) => {
                    ids.push(id.to_string());
                    if !is_milestone_id(id) {
                        errors.push(error(
                            &recipe_yaml,
                            &format!("{field_base}.id"),
                            "must match NN-name (e.g. 01-bootloader)",
                        ));
                    }
                }
                None if get(item, "id").is_some() => {
                    errors.push(error(
                        &recipe_yaml,
                        &format!("{field_base}.id"),
                        "must be a string",
                    ));
                }
                None => {}
            }

            if get(item, "title").is_some() && get_string(item, "title").is_none() {
                errors.push(error(
                    &recipe_yaml,
                    &format!("{field_base}.title"),
                    "must be a string",
                ));
            }
            if get(item, "demo").is_some() && get_string(item, "demo").is_none() {
                errors.push(error(
                    &recipe_yaml,
                    &format!("{field_base}.demo"),
                    "must be a string",
                ));
            }
            if let Some(prerequisites) = get(item, "prerequisites") {
                validate_string_array(
                    &mut errors,
                    &recipe_yaml,
                    &format!("{field_base}.prerequisites"),
                    prerequisites,
                    true,
                );
            }
            for key in ["goal", "verification_summary", "split_if_stuck"] {
                if get(item, key).is_some() && get_string(item, key).is_none() {
                    errors.push(error(
                        &recipe_yaml,
                        &format!("{field_base}.{key}"),
                        "must be a string",
                    ));
                }
            }
            if let Some(expected_artifacts) = get(item, "expected_artifacts") {
                validate_string_array(
                    &mut errors,
                    &recipe_yaml,
                    &format!("{field_base}.expected_artifacts"),
                    expected_artifacts,
                    true,
                );
            }
            if let Some(estimated_verify_minutes) = get(item, "estimated_verify_minutes") {
                match estimated_verify_minutes.as_u64() {
                    Some(0) => errors.push(error(
                        &recipe_yaml,
                        &format!("{field_base}.estimated_verify_minutes"),
                        "must be a positive integer",
                    )),
                    Some(_) => {}
                    None => errors.push(error(
                        &recipe_yaml,
                        &format!("{field_base}.estimated_verify_minutes"),
                        "must be a positive integer",
                    )),
                }
            }
        }

        let mut seen = std::collections::BTreeSet::new();
        for id in &ids {
            if !seen.insert(id.clone()) {
                errors.push(error(
                    &recipe_yaml,
                    "milestones",
                    &format!("duplicate milestone id '{id}'"),
                ));
            }
        }

        let mut last_number = None;
        let mut last_id = None::<String>;
        for id in ids.iter().filter(|id| is_milestone_id(id)) {
            let number = id
                .split_once('-')
                .and_then(|(prefix, _)| prefix.parse::<u32>().ok())
                .unwrap_or(0);
            if let Some(previous) = last_number
                && number <= previous
            {
                errors.push(error(
                    &recipe_yaml,
                    "milestones",
                    &format!(
                        "milestones must be ordered by numeric prefix: '{}' before '{}'",
                        last_id.as_deref().unwrap_or_default(),
                        id
                    ),
                ));
                break;
            }
            last_number = Some(number);
            last_id = Some(id.clone());
        }
    }

    errors
}

pub fn validate_milestones(recipe_target: &Path) -> Vec<String> {
    let recipe_dir = recipe_dir_path(recipe_target);
    let recipe_yaml = recipe_dir.join("recipe.yaml");
    let mut errors = Vec::new();

    let raw = match fs::read_to_string(&recipe_yaml) {
        Ok(raw) => raw,
        Err(_) => {
            errors.push(error(&recipe_yaml, "recipe.yaml", "file not found"));
            return errors;
        }
    };

    let doc: Value = match serde_yaml::from_str(&raw) {
        Ok(doc) => doc,
        Err(err) => {
            errors.push(error(
                &recipe_yaml,
                "recipe.yaml",
                &format!("invalid YAML: {err}"),
            ));
            return errors;
        }
    };

    let Some(root) = doc.as_mapping() else {
        errors.push(error(
            &recipe_yaml,
            "milestones",
            "must be a non-empty array before milestone validation",
        ));
        return errors;
    };
    let Some(milestones) = get(root, "milestones").and_then(Value::as_sequence) else {
        errors.push(error(
            &recipe_yaml,
            "milestones",
            "must be a non-empty array before milestone validation",
        ));
        return errors;
    };

    let mut declared_ids = Vec::new();
    for (index, item) in milestones.iter().enumerate() {
        match item
            .as_mapping()
            .and_then(|mapping| get_string(mapping, "id"))
            .map(str::to_string)
        {
            Some(id) => declared_ids.push(id),
            None => errors.push(error(
                &recipe_yaml,
                &format!("milestones[{index}].id"),
                "must be a string",
            )),
        }
    }

    let milestones_root = recipe_dir.join("milestones");
    if !milestones_root.exists() {
        errors.push(error(&milestones_root, "milestones", "directory not found"));
        return errors;
    }
    if !milestones_root.is_dir() {
        errors.push(error(&milestones_root, "milestones", "must be a directory"));
        return errors;
    }

    for milestone_id in &declared_ids {
        let milestone_dir = milestones_root.join(milestone_id);
        if !milestone_dir.exists() {
            errors.push(error(
                &milestone_dir,
                "milestone",
                "declared in recipe.yaml but directory is missing",
            ));
            continue;
        }
        if !milestone_dir.is_dir() {
            errors.push(error(&milestone_dir, "milestone", "must be a directory"));
            continue;
        }
        for required in REQUIRED_MILESTONE_FILES {
            let path = milestone_dir.join(required);
            if !path.exists() {
                errors.push(error(&path, "required-file", "missing"));
            }
        }

        let agent_path = milestone_dir.join("agent.md");
        if agent_path.is_file() {
            let agent = match fs::read_to_string(&agent_path) {
                Ok(agent) => agent,
                Err(err) => {
                    errors.push(error(
                        &agent_path,
                        "agent.md",
                        &format!("failed to read file: {err}"),
                    ));
                    continue;
                }
            };
            validate_agent_instructions(&agent_path, &agent, &mut errors);
        }
    }

    let actual_dirs = match fs::read_dir(&milestones_root) {
        Ok(entries) => entries,
        Err(err) => {
            errors.push(error(
                &milestones_root,
                "milestones",
                &format!("failed to read directory: {err}"),
            ));
            return errors;
        }
    };
    let declared_set: std::collections::BTreeSet<_> = declared_ids.iter().cloned().collect();
    for entry in actual_dirs.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !declared_set.contains(name) {
            errors.push(error(
                &path,
                "milestone",
                "directory exists but is not declared in recipe.yaml",
            ));
        }
    }

    errors
}

fn validate_agent_instructions(agent_path: &Path, agent_markdown: &str, errors: &mut Vec<String>) {
    let Some(learner) = extract_track_section(agent_markdown, "## Learner Track") else {
        errors.push(error(
            agent_path,
            "agent.md",
            "missing '## Learner Track' section",
        ));
        return;
    };
    let Some(builder) = extract_track_section(agent_markdown, "## Builder Track") else {
        errors.push(error(
            agent_path,
            "agent.md",
            "missing '## Builder Track' section",
        ));
        return;
    };

    let learner_lower = learner.to_ascii_lowercase();
    if !learner.contains('?') {
        errors.push(error(
            agent_path,
            "agent.md",
            "learner track must include at least one explicit question",
        ));
    }
    if !learner_lower.contains("explain") {
        errors.push(error(
            agent_path,
            "agent.md",
            "learner track must require explanation before or during implementation",
        ));
    }
    if !(learner_lower.contains("ask") || learner_lower.contains("question")) {
        errors.push(error(
            agent_path,
            "agent.md",
            "learner track must instruct the agent to ask the learner a question",
        ));
    }
    let builder_lower = builder.to_ascii_lowercase();
    if !builder_lower.contains("implement") {
        errors.push(error(
            agent_path,
            "agent.md",
            "builder track must instruct the agent to implement directly",
        ));
    }
    if !builder_lower.contains("check") {
        errors.push(error(
            agent_path,
            "agent.md",
            "builder track must require running milestone checks",
        ));
    }
}

fn extract_track_section<'a>(markdown: &'a str, heading: &str) -> Option<&'a str> {
    let start = markdown.find(heading)?;
    let rest = &markdown[start..];
    let end = rest
        .match_indices("\n## ")
        .map(|(index, _)| index)
        .find(|index| *index > 0)
        .unwrap_or(rest.len());
    Some(rest[..end].trim())
}

fn validate_string_array(
    errors: &mut Vec<String>,
    recipe_yaml: &Path,
    field: &str,
    value: &Value,
    require_non_empty: bool,
) {
    let Some(items) = value.as_sequence() else {
        errors.push(error(recipe_yaml, field, "must be a non-empty array"));
        return;
    };

    if require_non_empty && items.is_empty() {
        errors.push(error(recipe_yaml, field, "must be a non-empty array"));
        return;
    }

    for (index, item) in items.iter().enumerate() {
        if item.as_str().is_none_or(|text| text.trim().is_empty()) {
            errors.push(error(
                recipe_yaml,
                &format!("{field}[{index}]"),
                "must be a non-empty string",
            ));
        }
    }
}

fn get<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

fn get_string<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a str> {
    get(mapping, key)?.as_str()
}

fn error(path: &Path, field: &str, message: &str) -> String {
    format!("{}: {field}: {message}", path.display())
}

fn recipe_yaml_path(target: &Path) -> PathBuf {
    if target.is_dir() {
        target.join("recipe.yaml")
    } else {
        target.to_path_buf()
    }
}

fn recipe_dir_path(target: &Path) -> PathBuf {
    if target.is_dir() {
        target.to_path_buf()
    } else {
        target.parent().unwrap_or(target).to_path_buf()
    }
}

fn is_kebab_case(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        })
}

fn is_milestone_id(value: &str) -> bool {
    let Some((prefix, rest)) = value.split_once('-') else {
        return false;
    };
    prefix.len() == 2 && prefix.chars().all(|ch| ch.is_ascii_digit()) && is_kebab_case(rest)
}

#[cfg(test)]
mod tests {
    use super::{validate_milestones, validate_recipe, validate_recipe_yaml};
    use std::path::PathBuf;

    fn fixture(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("recipes")
            .join(path)
    }

    #[test]
    fn yaml_validator_accepts_valid_recipe() {
        assert!(validate_recipe_yaml(&fixture("valid-minimal")).is_empty());
    }

    #[test]
    fn yaml_validator_rejects_missing_required_field() {
        let errors = validate_recipe_yaml(&fixture("invalid-yaml-missing-required"));
        assert!(errors.iter().any(|error| error.contains("difficulty")));
    }

    #[test]
    fn yaml_validator_rejects_duplicate_milestone_id() {
        let errors = validate_recipe_yaml(&fixture("invalid-yaml-duplicate-milestone-id"));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate milestone id"))
        );
    }

    #[test]
    fn yaml_validator_rejects_invalid_milestone_metadata() {
        let errors = validate_recipe_yaml(&fixture("invalid-yaml-milestone-metadata"));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("verification_summary"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("expected_artifacts"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("estimated_verify_minutes"))
        );
    }

    #[test]
    fn milestone_validator_rejects_missing_file() {
        let errors = validate_milestones(&fixture("invalid-milestones-missing-file"));
        assert!(errors.iter().any(|error| error.contains("demo.sh")));
        assert!(errors.iter().any(|error| error.contains("missing")));
    }

    #[test]
    fn milestone_validator_rejects_invalid_agent_track_structure() {
        let errors = validate_milestones(&fixture("invalid-milestones-agent-track"));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("explicit question"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("running milestone checks"))
        );
    }

    #[test]
    fn unified_validator_accepts_valid_recipe() {
        assert!(validate_recipe(&fixture("valid-minimal")).is_ok());
    }
}
