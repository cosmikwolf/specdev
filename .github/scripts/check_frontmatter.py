#!/usr/bin/env python3
"""Fail when a skill or agent frontmatter does not parse as YAML.

Claude Code reads the `---` block at the top of `skills/*/SKILL.md` and
`agents/*.md` with a YAML parser. A block that does not parse drops the
skill from the session listing silently: `claude plugin validate` does not
read these files, and nothing else in the tree does either. On 2026-09-13
a co-worker's session saw 10 of 17 skills for exactly this reason - seven
descriptions carried a bare `: ` or an unquoted `[...]`, both of which YAML
rejects in a plain scalar.

Checks, per file:
  1. the file starts with a `---` line and has a closing `---`;
  2. the block parses with PyYAML;
  3. `name` and `description` are present and are strings;
  4. for skills, `name` equals the directory name.

Exit 0 when every file passes; exit 1 with one line per failure otherwise.
"""
from __future__ import annotations

import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]


def frontmatter(path: Path) -> str | None:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            return "\n".join(lines[1:i])
    return None


def check(path: Path, expect_name: str | None) -> list[str]:
    block = frontmatter(path)
    if block is None:
        return [f"{path}: no `---` frontmatter block"]
    try:
        data = yaml.safe_load(block)
    except yaml.YAMLError as e:
        first = str(e).splitlines()[0]
        return [f"{path}: frontmatter is not valid YAML: {first}"]
    problems: list[str] = []
    if not isinstance(data, dict):
        return [f"{path}: frontmatter is not a mapping"]
    for key in ("name", "description"):
        if not isinstance(data.get(key), str) or not data[key].strip():
            problems.append(f"{path}: `{key}` is missing or not a string")
    if expect_name and data.get("name") != expect_name:
        problems.append(
            f"{path}: `name` is {data.get('name')!r}, directory is {expect_name!r}"
        )
    return problems


def main() -> int:
    failures: list[str] = []
    seen = 0
    for skill in sorted((ROOT / "skills").glob("*/SKILL.md")):
        seen += 1
        failures += check(skill, skill.parent.name)
    for agent in sorted((ROOT / "agents").glob("*.md")):
        seen += 1
        failures += check(agent, None)
    for line in failures:
        print(line)
    if failures:
        print(f"{len(failures)} problem(s) in {seen} file(s)")
        return 1
    print(f"frontmatter ok: {seen} file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
