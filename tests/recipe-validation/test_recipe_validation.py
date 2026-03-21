#!/usr/bin/env python3
"""Tests for Epic 1 validators using fixture recipes."""

from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent.parent
FIXTURES = ROOT / "tests" / "fixtures" / "recipes"


def run_cmd(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


class RecipeValidationTests(unittest.TestCase):
    def test_yaml_validator_accepts_valid_recipe(self) -> None:
        recipe_dir = FIXTURES / "valid-minimal"
        result = run_cmd("python3", "scripts/validate-recipe-yaml.py", str(recipe_dir))
        self.assertEqual(result.returncode, 0, msg=result.stderr)

    def test_yaml_validator_rejects_missing_required_field(self) -> None:
        recipe_dir = FIXTURES / "invalid-yaml-missing-required"
        result = run_cmd("python3", "scripts/validate-recipe-yaml.py", str(recipe_dir))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("difficulty", result.stderr)

    def test_yaml_validator_rejects_duplicate_milestone_id(self) -> None:
        recipe_dir = FIXTURES / "invalid-yaml-duplicate-milestone-id"
        result = run_cmd("python3", "scripts/validate-recipe-yaml.py", str(recipe_dir))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicate milestone id", result.stderr)

    def test_yaml_validator_rejects_missing_milestone_prerequisites(self) -> None:
        recipe_dir = FIXTURES / "invalid-yaml-missing-milestone-prerequisites"
        result = run_cmd("python3", "scripts/validate-recipe-yaml.py", str(recipe_dir))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("prerequisites", result.stderr)

    def test_milestone_validator_accepts_valid_recipe(self) -> None:
        recipe_dir = FIXTURES / "valid-minimal"
        result = run_cmd("python3", "scripts/validate-milestones.py", str(recipe_dir))
        self.assertEqual(result.returncode, 0, msg=result.stderr)

    def test_milestone_validator_rejects_missing_file(self) -> None:
        recipe_dir = FIXTURES / "invalid-milestones-missing-file"
        result = run_cmd("python3", "scripts/validate-milestones.py", str(recipe_dir))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("demo.sh", result.stderr)
        self.assertIn("missing", result.stderr)

    def test_unified_validator_accepts_valid_recipe(self) -> None:
        recipe_dir = FIXTURES / "valid-minimal"
        result = run_cmd("scripts/validate-recipe", str(recipe_dir))
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        self.assertIn("validation passed", result.stdout)

    def test_unified_validator_fails_on_invalid_yaml(self) -> None:
        recipe_dir = FIXTURES / "invalid-yaml-missing-required"
        result = run_cmd("scripts/validate-recipe", str(recipe_dir))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("difficulty", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
