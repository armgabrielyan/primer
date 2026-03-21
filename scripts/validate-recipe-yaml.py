#!/usr/bin/env python3
"""Validate recipe.yaml against the v0.1 contract."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except Exception as exc:  # pragma: no cover
    print(f"dependency: PyYAML import failed: {exc}", file=sys.stderr)
    sys.exit(2)


DIFFICULTY_VALUES = {"easy", "medium", "hard", "expert"}
RECIPE_ID_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
MILESTONE_ID_RE = re.compile(r"^[0-9]{2}-[a-z0-9]+(?:-[a-z0-9]+)*$")


def error(path: Path, field: str, message: str) -> str:
    return f"{path}: {field}: {message}"


def load_yaml(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def as_yaml_path(arg: str) -> Path:
    p = Path(arg)
    if p.is_dir():
        return p / "recipe.yaml"
    return p


def validate(path: Path) -> list[str]:
    errs: list[str] = []
    if not path.exists():
        return [error(path, "recipe.yaml", "file not found")]

    try:
        doc = load_yaml(path)
    except Exception as exc:
        return [error(path, "recipe.yaml", f"invalid YAML: {exc}")]

    if not isinstance(doc, dict):
        return [error(path, "<root>", "expected mapping/object")]

    required_fields = [
        "id",
        "title",
        "description",
        "difficulty",
        "stack",
        "tracks",
        "milestones",
    ]
    for key in required_fields:
        if key not in doc:
            errs.append(error(path, key, "is required"))

    recipe_id = doc.get("id")
    if isinstance(recipe_id, str):
        if not RECIPE_ID_RE.match(recipe_id):
            errs.append(error(path, "id", "must be kebab-case (e.g. operating-system)"))
    elif recipe_id is not None:
        errs.append(error(path, "id", "must be a string"))

    difficulty = doc.get("difficulty")
    if difficulty is not None and difficulty not in DIFFICULTY_VALUES:
        errs.append(
            error(
                path,
                "difficulty",
                f"must be one of {sorted(DIFFICULTY_VALUES)}",
            )
        )

    stack = doc.get("stack")
    if stack is not None:
        if not isinstance(stack, dict):
            errs.append(error(path, "stack", "must be an object"))
        else:
            for key in ["id", "label", "tools"]:
                if key not in stack:
                    errs.append(error(path, f"stack.{key}", "is required"))
            stack_id = stack.get("id")
            if stack_id is not None and not isinstance(stack_id, str):
                errs.append(error(path, "stack.id", "must be a string"))
            label = stack.get("label")
            if label is not None and not isinstance(label, str):
                errs.append(error(path, "stack.label", "must be a string"))
            tools = stack.get("tools")
            if tools is not None:
                if not isinstance(tools, list) or not tools:
                    errs.append(error(path, "stack.tools", "must be a non-empty array"))
                else:
                    for i, tool in enumerate(tools):
                        if not isinstance(tool, str) or not tool.strip():
                            errs.append(
                                error(path, f"stack.tools[{i}]", "must be a non-empty string")
                            )

    if "stacks" in doc:
        errs.append(error(path, "stacks", "is not supported in v0.1; use singular 'stack'"))

    tracks = doc.get("tracks")
    if tracks is not None:
        if not isinstance(tracks, dict):
            errs.append(error(path, "tracks", "must be an object"))
        else:
            for key in ["learner", "builder"]:
                if key not in tracks:
                    errs.append(error(path, f"tracks.{key}", "is required"))
                elif not isinstance(tracks[key], dict):
                    errs.append(error(path, f"tracks.{key}", "must be an object"))
                elif "description" not in tracks[key] or not isinstance(
                    tracks[key].get("description"), str
                ):
                    errs.append(error(path, f"tracks.{key}.description", "must be a string"))

    milestones = doc.get("milestones")
    if milestones is not None:
        if not isinstance(milestones, list) or not milestones:
            errs.append(error(path, "milestones", "must be a non-empty array"))
        else:
            ids: list[str] = []
            for i, item in enumerate(milestones):
                field_base = f"milestones[{i}]"
                if not isinstance(item, dict):
                    errs.append(error(path, field_base, "must be an object"))
                    continue
                for key in ["id", "title", "demo", "prerequisites"]:
                    if key not in item:
                        errs.append(error(path, f"{field_base}.{key}", "is required"))

                ms_id = item.get("id")
                if isinstance(ms_id, str):
                    ids.append(ms_id)
                    if not MILESTONE_ID_RE.match(ms_id):
                        errs.append(
                            error(path, f"{field_base}.id", "must match NN-name (e.g. 01-bootloader)")
                        )
                elif ms_id is not None:
                    errs.append(error(path, f"{field_base}.id", "must be a string"))

                title = item.get("title")
                if title is not None and not isinstance(title, str):
                    errs.append(error(path, f"{field_base}.title", "must be a string"))

                demo = item.get("demo")
                if demo is not None and not isinstance(demo, str):
                    errs.append(error(path, f"{field_base}.demo", "must be a string"))

                prerequisites = item.get("prerequisites")
                if prerequisites is not None:
                    if not isinstance(prerequisites, list) or not prerequisites:
                        errs.append(
                            error(path, f"{field_base}.prerequisites", "must be a non-empty array")
                        )
                    else:
                        for j, tool in enumerate(prerequisites):
                            if not isinstance(tool, str) or not tool.strip():
                                errs.append(
                                    error(
                                        path,
                                        f"{field_base}.prerequisites[{j}]",
                                        "must be a non-empty string",
                                    )
                                )

            seen = set()
            for ms_id in ids:
                if ms_id in seen:
                    errs.append(error(path, "milestones", f"duplicate milestone id '{ms_id}'"))
                seen.add(ms_id)

            nums: list[tuple[int, str]] = []
            for ms_id in ids:
                if MILESTONE_ID_RE.match(ms_id):
                    nums.append((int(ms_id.split("-", 1)[0]), ms_id))
            for idx in range(1, len(nums)):
                if nums[idx][0] <= nums[idx - 1][0]:
                    errs.append(
                        error(
                            path,
                            "milestones",
                            f"milestones must be ordered by numeric prefix: '{nums[idx - 1][1]}' before '{nums[idx][1]}'",
                        )
                    )
                    break

    return errs


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: scripts/validate-recipe-yaml.py <recipe-dir|recipe.yaml>", file=sys.stderr)
        return 2

    yaml_path = as_yaml_path(sys.argv[1])
    errs = validate(yaml_path)
    if errs:
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
