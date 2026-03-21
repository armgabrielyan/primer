#!/usr/bin/env python3
"""Generate Claude Code adapter artifacts from recipe + shared contracts."""

from __future__ import annotations

import argparse
import shutil
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
REQUIRED_SHARED = ["next-milestone.md", "check.md", "explain.md", "status.md", "build.md"]
COMMAND_NAME_MAP = {
    "build.md": "primer-build.md",
    "check.md": "primer-check.md",
    "explain.md": "primer-explain.md",
    "status.md": "primer-status.md",
    "next-milestone.md": "primer-next-milestone.md",
}


def load_yaml(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Generate CLAUDE.md and .claude/commands from a recipe."
    )
    p.add_argument("recipe_dir", help="Path to recipe directory (contains recipe.yaml)")
    p.add_argument(
        "--output-dir",
        default=".",
        help="Project root where CLAUDE.md and .claude/commands will be written",
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

    claude_md = output_dir / "CLAUDE.md"
    commands_dir = output_dir / ".claude" / "commands"
    commands_dir.mkdir(parents=True, exist_ok=True)

    claude_md.write_text(
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

    for filename in REQUIRED_SHARED:
        src = SHARED_DIR / filename
        if not src.exists():
            raise ValueError(f"{src}: required shared command definition missing")
        shutil.copyfile(src, commands_dir / COMMAND_NAME_MAP[filename])


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
