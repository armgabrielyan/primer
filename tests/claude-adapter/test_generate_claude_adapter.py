#!/usr/bin/env python3
"""Tests for Claude adapter generation."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent.parent
RECIPE_DIR = ROOT / "recipes" / "operating-system"
SHARED_DIR = ROOT / "adapters" / "_shared"


def run_cmd(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


class ClaudeAdapterGenerationTests(unittest.TestCase):
    def test_generation_creates_expected_files(self) -> None:
        with tempfile.TemporaryDirectory(prefix="primer-claude-gen-") as tmp:
            out = Path(tmp)
            result = run_cmd(
                "scripts/generate-claude-adapter",
                str(RECIPE_DIR),
                "--output-dir",
                str(out),
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr)

            self.assertTrue((out / "CLAUDE.md").exists())
            self.assertTrue((out / ".claude" / "commands" / "primer-build.md").exists())
            self.assertTrue((out / ".claude" / "commands" / "primer-next-milestone.md").exists())
            self.assertTrue((out / ".claude" / "commands" / "primer-check.md").exists())
            self.assertTrue((out / ".claude" / "commands" / "primer-explain.md").exists())
            self.assertTrue((out / ".claude" / "commands" / "primer-status.md").exists())

    def test_state_block_defaults_and_recipe_path(self) -> None:
        with tempfile.TemporaryDirectory(prefix="primer-claude-gen-") as tmp:
            out = Path(tmp)
            result = run_cmd(
                "scripts/generate-claude-adapter",
                str(RECIPE_DIR),
                "--output-dir",
                str(out),
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr)

            content = read(out / "CLAUDE.md")
            self.assertIn("primer_state:", content)
            self.assertIn("recipe_id: operating-system", content)
            self.assertIn(f"recipe_path: {RECIPE_DIR.as_posix()}", content)
            self.assertIn(f"workspace_root: {out.resolve().as_posix()}", content)
            self.assertIn("milestone_id: 01-bootloader", content)
            self.assertIn("verified_milestone_id: null", content)
            self.assertIn("track: learner", content)
            self.assertIn("stack_id: c-x86", content)
            self.assertIn("`primer-build`", content)

    def test_track_and_milestone_overrides(self) -> None:
        with tempfile.TemporaryDirectory(prefix="primer-claude-gen-") as tmp:
            out = Path(tmp)
            result = run_cmd(
                "scripts/generate-claude-adapter",
                str(RECIPE_DIR),
                "--output-dir",
                str(out),
                "--track",
                "builder",
                "--milestone-id",
                "03-vga-output",
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr)
            content = read(out / "CLAUDE.md")
            self.assertIn("track: builder", content)
            self.assertIn("milestone_id: 03-vga-output", content)

    def test_shared_commands_are_copied_exactly(self) -> None:
        with tempfile.TemporaryDirectory(prefix="primer-claude-gen-") as tmp:
            out = Path(tmp)
            result = run_cmd(
                "scripts/generate-claude-adapter",
                str(RECIPE_DIR),
                "--output-dir",
                str(out),
            )
            self.assertEqual(result.returncode, 0, msg=result.stderr)

            generated_names = {
                "build.md": "primer-build.md",
                "next-milestone.md": "primer-next-milestone.md",
                "check.md": "primer-check.md",
                "explain.md": "primer-explain.md",
                "status.md": "primer-status.md",
            }
            for name in ["build.md", "next-milestone.md", "check.md", "explain.md", "status.md"]:
                generated = read(out / ".claude" / "commands" / generated_names[name])
                shared = read(SHARED_DIR / name)
                self.assertEqual(generated, shared, msg=f"mismatch in {name}")

    def test_invalid_milestone_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="primer-claude-gen-") as tmp:
            out = Path(tmp)
            result = run_cmd(
                "scripts/generate-claude-adapter",
                str(RECIPE_DIR),
                "--output-dir",
                str(out),
                "--milestone-id",
                "99-nope",
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("is not declared in milestones", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
