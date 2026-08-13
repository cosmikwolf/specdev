#!/usr/bin/env python3
"""Freshness sweep: has the world moved past what our units pin?

Every block and bridge pins upstream versions in `codegen/<lang>/deps.yaml`, and
until now nothing noticed when a pin went stale. That is not a drift problem —
the pin was written down once and everything downstream is verified rigorously
*against* it, so rigor stays anchored to a bet nobody re-checks. It bit on a
library that released 1.0 and renamed its core vocabulary, leaving prose that
compiles nowhere.

WHAT THIS REPORTS, AND WHY THE DISTINCTION IS THE WHOLE POINT

    current               pinned version is the latest              silent
    held                  behind on purpose, with a reason on file  silent
    behind, compiles      newer exists; the reference still builds  informational
    behind, BREAKS        newer exists; the reference fails on it   actionable

Only the last is worth waking someone for, and only it sets a nonzero exit.
A report that cries wolf on every run gets muted, and a muted gate is worse
than no gate — which is why deliberate holds are read from a policy file and
reported as held rather than as findings.

WHAT IT DOES NOT DO

It never edits `deps.yaml`, and it must never write the honesty status a unit
declares about its own prose. Those answer different questions: a status field
is an author's claim that a unit's prose has a compiled witness against its
pin; this is a sensor reading whether the world moved past that pin. A unit
pinned at 0.36 stays honestly verified when 0.37 ships and breaks — nothing
about its witness changed, there is only an upgrade cost. Conflating them would
also make a scheduled job a writer of spec content that gets inlined into user
projects.

It sweeps `deps.yaml` pins only, not reference-manifest pins. A reference may
deliberately carry an old version to stand in for a host that has not moved
(two renamed copies of one crate, say), and flagging those would be permanent
noise. The uncovered case reaches us anyway, in the right order: when the host
crate goes behind, this sweep flags the host, and the composition gate catches
the reference mismatch the moment someone bumps it.

Requires PyYAML and network access to crates.io, pub.dev, and any git remote a
unit pins.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path

import yaml

USER_AGENT = "specdev-freshness-sweep (plugin CI)"
TIMEOUT = 30


# ---------------------------------------------------------------------------
# registry lookups
# ---------------------------------------------------------------------------


def _get(url: str, attempts: int = 3) -> str:
    """Fetch with retry.

    A scheduled job that dies on a transient reset reports nothing, which is
    indistinguishable from "nothing to report" — the failure mode this whole
    script exists to avoid. Observed live: the registry reset a connection
    mid-sweep and took the run down with it.
    """
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
                return resp.read().decode("utf-8")
        except urllib.error.HTTPError:
            raise  # a 404 is an answer, not a flake
        except (urllib.error.URLError, ConnectionError, TimeoutError):
            if attempt == attempts - 1:
                raise
            time.sleep(2 * (attempt + 1))
    raise RuntimeError("unreachable")


def _index_path(name: str) -> str:
    """Sparse-index path for a crate, per cargo's own layout rules."""
    n = name.lower()
    if len(n) == 1:
        return f"1/{n}"
    if len(n) == 2:
        return f"2/{n}"
    if len(n) == 3:
        return f"3/{n[0]}/{n}"
    return f"{n[:2]}/{n[2:4]}/{n}"


def _version_key(v: str) -> tuple:
    """Sort key for a semver string. Prerelease sorts below its release."""
    core, _, pre = v.partition("-")
    parts = []
    for piece in core.split("."):
        try:
            parts.append(int(piece))
        except ValueError:
            parts.append(0)
    while len(parts) < 3:
        parts.append(0)
    # A release outranks any prerelease of the same core version.
    return (parts[0], parts[1], parts[2], 1 if not pre else 0, pre)


def latest_crates_io(name: str) -> str | None:
    """Highest non-yanked, non-prerelease version on crates.io."""
    try:
        body = _get(f"https://index.crates.io/{_index_path(name)}")
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return None
        raise
    versions = []
    for line in body.splitlines():
        line = line.strip()
        if not line:
            continue
        entry = json.loads(line)
        if entry.get("yanked"):
            continue
        if "-" in entry["vers"]:  # prerelease
            continue
        versions.append(entry["vers"])
    return max(versions, key=_version_key) if versions else None


def latest_pub_dev(name: str) -> str | None:
    try:
        body = _get(f"https://pub.dev/api/packages/{name}")
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return None
        raise
    return json.loads(body).get("latest", {}).get("version")


def remote_head(url: str) -> str | None:
    out = subprocess.run(
        ["git", "ls-remote", url, "HEAD"],
        capture_output=True,
        text=True,
        timeout=TIMEOUT,
    )
    if out.returncode != 0:
        return None
    return out.stdout.split()[0] if out.stdout.split() else None


# ---------------------------------------------------------------------------
# reading the tree
# ---------------------------------------------------------------------------


@dataclass
class Pin:
    unit: str  # "blocks/eframe"
    language: str  # "rust" | "dart"
    name: str  # crate / package name
    pinned: str  # declared version or git rev
    kind: str  # "registry" | "git"
    git_url: str | None = None


def read_pins(root: Path) -> list[Pin]:
    pins: list[Pin] = []
    for kind_dir in ("blocks", "bridges"):
        for deps in sorted((root / kind_dir).glob("*/codegen/*/deps.yaml")):
            unit = f"{kind_dir}/{deps.parents[2].name}"
            language = deps.parent.name
            doc = yaml.safe_load(deps.read_text()) or {}
            for entry in doc.get("crates") or []:
                source = entry.get("source")
                if isinstance(source, dict) and "git" in source:
                    pins.append(
                        Pin(
                            unit,
                            language,
                            entry["name"],
                            str(source.get("rev", "(unpinned HEAD)")),
                            "git",
                            source["git"],
                        )
                    )
                else:
                    pins.append(
                        Pin(unit, language, entry["name"], str(entry.get("version", "")), "registry")
                    )
            # Dart manifests use a different section and key: `pub:` with
            # `package:` rather than `crates:` with `name:`.
            for entry in doc.get("pub") or []:
                pins.append(
                    Pin(unit, language, entry["package"], str(entry.get("version", "")), "registry")
                )

    pins.extend(read_profile_pins(root))
    return pins


def read_profile_pins(root: Path) -> list[Pin]:
    """Versions pinned in a profile's architecture template.

    Profiles pin crates too — a panic handler, a logger backend, a runtime —
    and those reach every project bootstrapped from the profile. Sweeping only
    block and bridge manifests would report a confident all-current while
    shipped profile content sat a major version behind, which is a false clean
    and worse than no report.

    Profiles ship no reference crate, so the build-against-latest leg cannot
    answer "does it break" for these. They are reported as behind and NOT
    built, said plainly, rather than being quietly counted as compiling.

    Templates are prose to the analyzer but are valid TOML, so they are parsed
    rather than pattern-matched. Any table carrying both `crate` and `version`
    is a pin, wherever it sits — no profile-specific key names.
    """
    pins: list[Pin] = []
    for template in sorted(root.glob("profiles/*/architecture.toml.template")):
        profile = f"profiles/{template.parent.name}"
        try:
            doc = tomllib.loads(template.read_text())
        except tomllib.TOMLDecodeError as e:
            print(f"warning: {profile} architecture template does not parse: {e}", file=sys.stderr)
            continue
        for crate, version in _walk_crate_refs(doc):
            pins.append(Pin(profile, "rust", crate, version, "registry"))
    return pins


def _walk_crate_refs(node) -> list[tuple[str, str]]:
    found: list[tuple[str, str]] = []
    if isinstance(node, dict):
        if isinstance(node.get("crate"), str) and isinstance(node.get("version"), str):
            found.append((node["crate"], node["version"]))
        for value in node.values():
            found.extend(_walk_crate_refs(value))
    elif isinstance(node, list):
        for value in node:
            found.extend(_walk_crate_refs(value))
    return found


def satisfies(req: str, version: str) -> bool:
    """Does `version` satisfy the caret requirement `req`?

    This is what decides "behind", and getting it wrong in the lenient
    direction hides real staleness while getting it wrong in the strict
    direction cries wolf on every patch release — which gets the report muted.

    A caret pin already resolves to the newest compatible release, so being
    behind only means something when the latest version falls OUTSIDE the
    declared range: a major bump (or, below 1.0, a minor bump), which is
    exactly the class that needs a human. `eframe = "0.36"` is not stale
    because 0.36.1 exists; it is stale when 0.37 ships.
    """
    r = _components(req.lstrip("^~= "))
    v = _components(version)
    if v < r:
        return False
    if r[0] > 0:
        return v[0] == r[0]
    if r[1] > 0:
        return v[0] == 0 and v[1] == r[1]
    return v[0] == 0 and v[1] == 0 and v[2] == r[2]


def _components(v: str) -> tuple[int, int, int]:
    core = v.partition("-")[0].strip()
    parts = []
    for piece in core.split(".")[:3]:
        try:
            parts.append(int(piece))
        except ValueError:
            parts.append(0)
    while len(parts) < 3:
        parts.append(0)
    return (parts[0], parts[1], parts[2])


def read_policy(root: Path, unit: str, language: str) -> dict:
    """Deliberate holds, keyed by crate name.

    Lives under `reference/<lang>/pin-policy.yaml` — plugin-author state, so it
    must NOT sit in `deps.yaml`, which is inlined into every user project. Same
    placement reasoning that keeps reference-case declarations out of user
    trees: when a version is held back and why is a fact about maintaining this
    plugin, not about the project consuming it.
    """
    path = root / unit / "reference" / language / "pin-policy.yaml"
    if not path.is_file():
        return {}
    doc = yaml.safe_load(path.read_text()) or {}
    return {h["crate"]: h for h in doc.get("held") or []}


# ---------------------------------------------------------------------------
# does the newer version actually break us?
# ---------------------------------------------------------------------------


def _declaration_index(lines: list[str], crate: str) -> int | None:
    """Which dependency line should move to the newer version.

    Only the declaration keyed by the crate's own name. A renamed key
    (`egui_legacy = { package = "egui", ... }`) exists precisely to hold a
    DIFFERENT version — a unit standing in for two hosts that straddle an
    upstream break declares the same crate twice on purpose, and moving that
    second copy would report a break the unit already accounts for. The renamed
    key is used only when it is the sole declaration.
    """
    renamed = None
    for i, line in enumerate(lines):
        s = line.strip()
        if s.startswith("#") or "=" not in s:
            continue
        if s.split("=")[0].strip() == crate:
            return i
        if f'package = "{crate}"' in s and renamed is None:
            renamed = i
    return renamed


def build_against(root: Path, unit: str, language: str, crate: str, version: str) -> tuple[bool, str]:
    """Copy the unit's reference crate, repoint one dep at `version`, build.

    Returns (ok, detail). A unit with no reference crate cannot answer the
    question — reported as unknown rather than as a pass, because "we did not
    check" and "it works" are the two things a gate must never conflate.
    """
    ref = root / unit / "reference" / language
    manifest = ref / "Cargo.toml"
    if not manifest.is_file():
        return (True, "no reference crate — not checked")

    with tempfile.TemporaryDirectory() as tmp:
        dst = Path(tmp) / "ref"
        shutil.copytree(ref, dst, ignore=shutil.ignore_patterns("target"))
        lines = (dst / "Cargo.toml").read_text().splitlines()
        idx = _declaration_index(lines, crate)
        if idx is None:
            return (True, f"{crate} not declared in the reference — not checked")

        stripped = lines[idx].strip()
        key = stripped.split("=")[0].strip()
        if "{" in stripped:
            lines[idx] = re.sub(r'version\s*=\s*"[^"]*"', f'version = "{version}"', lines[idx])
        else:
            lines[idx] = f'{key} = "{version}"'
        (dst / "Cargo.toml").write_text("\n".join(lines) + "\n")
        (dst / "Cargo.lock").unlink(missing_ok=True)

        proc = subprocess.run(
            ["cargo", "build", "--quiet"], cwd=dst, capture_output=True, text=True
        )
        if proc.returncode == 0:
            return (True, "reference builds against it")
        first = next(
            (ln for ln in proc.stderr.splitlines() if ln.startswith("error")),
            proc.stderr.strip().splitlines()[0] if proc.stderr.strip() else "build failed",
        )
        return (False, first[:160])


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------


# `satisfies` decides every verdict this script prints, so it is the one piece
# worth proving rather than asserting. Wrong in the lenient direction it hides
# real staleness; wrong in the strict direction it cries wolf and gets the whole
# report muted. Runs offline, in CI, before the network work.
SATISFIES_CASES = [
    ("0.36", "0.36.1", True, "patch release within a 0.x pin"),
    ("0.36", "0.37.0", False, "below 1.0 a minor bump is a breaking bump"),
    ("1", "1.53.1", True, "a bare major pin admits any 1.x"),
    ("1", "2.0.0", False, "major bump"),
    ("^2.1.0", "2.1.1", True, "explicit caret, patch above the floor"),
    ("^2.1.0", "2.9.0", True, "explicit caret, minor above the floor"),
    ("^2.1.0", "3.0.0", False, "explicit caret, major bump"),
    ("1.0.3", "1.0.3", True, "exact match"),
    ("1.0.3", "1.0.2", False, "latest is below the declared floor"),
    ("0.0.5", "0.0.5", True, "0.0.x pin, unchanged"),
    ("0.0.5", "0.0.6", False, "0.0.x — every patch is a breaking bump"),
]


def self_test() -> int:
    bad = 0
    for req, ver, want, why in SATISFIES_CASES:
        got = satisfies(req, ver)
        if got != want:
            bad += 1
        print(f"{'ok  ' if got == want else 'FAIL'} satisfies({req!r}, {ver!r}) = {got} — {why}")
    print(f"\n{len(SATISFIES_CASES) - bad}/{len(SATISFIES_CASES)} passed")
    return 1 if bad else 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--plugin", type=Path, default=Path(__file__).resolve().parents[2])
    ap.add_argument(
        "--no-build",
        action="store_true",
        help="skip the build-against-latest leg; report behind/current only",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="check the version-satisfaction rule against its case table and exit",
    )
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    root: Path = args.plugin

    if not (root / ".claude-plugin" / "plugin.json").is_file():
        print(f"error: {root} is not a plugin tree", file=sys.stderr)
        return 2

    pins = read_pins(root)
    print(f"freshness sweep over {len(pins)} pinned dependencies in {root}\n")

    rows, actionable, checked = [], [], 0
    for pin in pins:
        policy = read_policy(root, pin.unit, pin.language).get(pin.name)

        if pin.kind == "git":
            head = remote_head(pin.git_url or "")
            current = head[:12] if head else "?"
            pinned_short = pin.pinned[:12]
            behind = bool(head) and not pin.pinned.startswith(head[:12])
        else:
            lookup = latest_pub_dev if pin.language == "dart" else latest_crates_io
            current = lookup(pin.name) or "?"
            pinned_short = pin.pinned
            behind = current != "?" and not satisfies(pin.pinned, current)

        if not behind:
            rows.append((pin, pinned_short, current, "current", ""))
            continue
        if policy:
            rows.append((pin, pinned_short, current, "held", policy.get("reason", "")))
            continue

        if args.no_build or pin.language != "rust":
            rows.append((pin, pinned_short, current, "behind", "not built"))
            continue

        checked += 1
        ok, detail = build_against(root, pin.unit, pin.language, pin.name, current)
        if ok:
            rows.append((pin, pinned_short, current, "behind", detail))
        else:
            rows.append((pin, pinned_short, current, "BREAKS", detail))
            actionable.append((pin, current, detail))

    width = max((len(p.unit) for p, *_ in rows), default=10)
    for pin, pinned, current, state, note in rows:
        marker = {"current": "  ", "held": "· ", "behind": "→ ", "BREAKS": "✗ "}[state]
        line = f"{marker}{pin.unit:<{width}}  {pin.name:<26} {pinned:<14} {current:<14} {state}"
        print(line + (f"   ({note})" if note else ""))

    print()
    counts = {s: sum(1 for *_, st, _ in rows if st == s) for s in ("current", "held", "behind", "BREAKS")}
    print(
        f"current: {counts['current']}   held: {counts['held']}   "
        f"behind: {counts['behind']}   BREAKS: {counts['BREAKS']}"
    )
    if args.no_build:
        print("(--no-build: nothing was built, so 'behind' here does not mean 'still compiles')")
    elif checked == 0:
        print("(nothing was behind, so no build-against-latest was attempted)")

    # A "behind" that could not be built is not the same as one that was built
    # and passed. Saying "nothing actionable" over the first kind would be the
    # report claiming a verification it never performed.
    unverified = [
        (p, cur, note)
        for p, _, cur, st, note in rows
        if st == "behind" and ("not checked" in note or "not built" in note)
    ]
    if unverified:
        print(
            f"\nUNVERIFIED — {len(unverified)} behind, with nothing able to say whether it breaks:"
        )
        for pin, current, note in unverified:
            print(f"  {pin.unit} / {pin.name}: {pin.pinned} → {current}   ({note})")
        print(
            "\nA profile ships no reference crate, so its pins have no compiled witness at\n"
            "all — the least-checked substrate in the tree. A major-version gap here wants\n"
            "a human, even though nothing can turn it red."
        )

    if not actionable:
        if not unverified:
            print("\nNothing actionable. Being behind is only a finding when it breaks.")
        return 0

    print("\nACTIONABLE — newer version exists and the reference fails against it:")
    for pin, current, detail in actionable:
        print(f"  {pin.unit} / {pin.name}: {pin.pinned} → {current}")
        print(f"      {detail}")
    print(
        "\nEither bump the pin and fix the prose the reference certifies, or record\n"
        "a deliberate hold in the unit's reference/<lang>/pin-policy.yaml with a reason."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
