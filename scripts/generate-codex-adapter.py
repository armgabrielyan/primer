#!/usr/bin/env python3
"""Generate Codex adapter artifacts from recipe + shared contracts."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

from adapter_context import render_adapter_context

try:
    import yaml
except Exception as exc:  # pragma: no cover
    print(f"dependency: PyYAML import failed: {exc}", file=sys.stderr)
    sys.exit(2)


ROOT = Path(__file__).resolve().parent.parent
SHARED_DIR = ROOT / "adapters" / "_shared"
SKILL_FILES = ["build.md", "check.md", "explain.md", "status.md", "next-milestone.md"]


def load_yaml(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Generate AGENTS.md and .agents/skills repo-local skills from a recipe."
    )
    p.add_argument("recipe_dir", help="Path to recipe directory (contains recipe.yaml)")
    p.add_argument(
        "--output-dir",
        default=".",
        help="Project root where AGENTS.md and .agents/skills will be written",
    )
    p.add_argument(
        "--track",
        default="learner",
        choices=["learner", "builder"],
        help="Initial track in generated state block",
    )
    p.add_argument(
        "--milestone-id",
        default=None,
        help="Initial milestone id. Defaults to first milestone in recipe.yaml",
    )
    return p.parse_args()


def validate_recipe(doc: Any, recipe_yaml: Path) -> tuple[str, str, list[str], str]:
    if not isinstance(doc, dict):
        raise ValueError(f"{recipe_yaml}: expected YAML object at root")
    recipe_id = doc.get("id")
    title = doc.get("title")
    stack = doc.get("stack")
    milestones = doc.get("milestones")

    if not isinstance(recipe_id, str) or not recipe_id.strip():
        raise ValueError(f"{recipe_yaml}: id: required non-empty string")
    if not isinstance(title, str) or not title.strip():
        raise ValueError(f"{recipe_yaml}: title: required non-empty string")
    if not isinstance(stack, dict):
        raise ValueError(f"{recipe_yaml}: stack: required object")
    stack_id = stack.get("id")
    if not isinstance(stack_id, str) or not stack_id.strip():
        raise ValueError(f"{recipe_yaml}: stack.id: required non-empty string")
    if not isinstance(milestones, list) or not milestones:
        raise ValueError(f"{recipe_yaml}: milestones: required non-empty array")

    milestone_ids: list[str] = []
    for i, m in enumerate(milestones):
        if not isinstance(m, dict) or not isinstance(m.get("id"), str):
            raise ValueError(f"{recipe_yaml}: milestones[{i}].id: required string")
        milestone_ids.append(m["id"])
    return recipe_id, title, milestone_ids, stack_id


def render_skill_md(*, skill_name: str, description: str, body: str) -> str:
    return f"""---
name: {skill_name}
description: {description}
---

{body}
"""


def render_openai_yaml(*, display_name: str, short_description: str, default_prompt: str) -> str:
    return f"""interface:
  display_name: "{display_name}"
  short_description: "{short_description}"
  default_prompt: "{default_prompt}"
"""


def build_skill_metadata(recipe_id: str, recipe_title: str, shared_filename: str) -> tuple[str, str, str, str]:
    command_name = shared_filename.removesuffix(".md")
    skill_name = f"primer-{command_name}"
    display_name = f"{recipe_title}: {command_name.replace('-', ' ').title()}"
    short_description = f"{command_name.replace('-', ' ').capitalize()} for the current Primer recipe"
    description = (
        f"Use when the user wants to {command_name.replace('-', ' ')} for the current Primer recipe in this repo workspace."
    )
    default_prompt = f"Use ${skill_name} for the current recipe milestone."
    return skill_name, display_name, short_description, description, default_prompt


def generate(recipe_dir: Path, output_dir: Path, track: str, milestone_id: str | None) -> None:
    recipe_dir = recipe_dir.resolve()
    output_dir = output_dir.resolve()
    recipe_yaml = recipe_dir / "recipe.yaml"
    if not recipe_yaml.exists():
        raise ValueError(f"{recipe_yaml}: file not found")
    doc = load_yaml(recipe_yaml)
    recipe_id, recipe_title, milestone_ids, stack_id = validate_recipe(doc, recipe_yaml)

    initial_milestone = milestone_id if milestone_id else milestone_ids[0]
    if initial_milestone not in milestone_ids:
        raise ValueError(
            f"{recipe_yaml}: milestone_id '{initial_milestone}' is not declared in milestones"
        )

    agents_md = output_dir / "AGENTS.md"
    skills_root = output_dir / ".agents" / "skills"
    skills_root.mkdir(parents=True, exist_ok=True)

    agents_md.write_text(
        render_adapter_context(
            recipe_title=recipe_title,
            recipe_id=recipe_id,
            recipe_path=recipe_dir,
            workspace_root=output_dir,
            milestone_id=initial_milestone,
            track=track,
            stack_id=stack_id,
        ),
        encoding="utf-8",
    )

    for filename in SKILL_FILES:
        src = SHARED_DIR / filename
        if not src.exists():
            raise ValueError(f"{src}: required shared command definition missing")
        body = src.read_text(encoding="utf-8")
        skill_name, display_name, short_description, description, default_prompt = build_skill_metadata(
            recipe_id, recipe_title, filename
        )
        skill_dir = skills_root / skill_name
        skill_agents_dir = skill_dir / "agents"
        skill_agents_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "SKILL.md").write_text(
            render_skill_md(skill_name=skill_name, description=description, body=body),
            encoding="utf-8",
        )
        (skill_agents_dir / "openai.yaml").write_text(
            render_openai_yaml(
                display_name=display_name,
                short_description=short_description,
                default_prompt=default_prompt,
            ),
            encoding="utf-8",
        )


def main() -> int:
    args = parse_args()
    try:
        generate(
            recipe_dir=Path(args.recipe_dir),
            output_dir=Path(args.output_dir),
            track=args.track,
            milestone_id=args.milestone_id,
        )
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
