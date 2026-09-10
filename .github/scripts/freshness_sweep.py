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
    ahead                 the feed's newest is older than the pin   silent
    unswept               no release feed for this language / SDK   informational
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


def _index_name_candidates(name: str) -> list[str]:
    """The names to try in the sparse index, in order.

    The index stores a crate under the name it was **published** with, but a
    manifest may spell it the other way round and cargo still resolves it:
    measured, `cargo add static-cell` adds `static_cell v2.1.1`. Asking the
    index verbatim therefore 404s on a legal pin — `profiles/firmware-layered`
    offers `crate = "static-cell"`, which is correct for cargo and absent from
    `st/at/static-cell` (#125). Try the spelling as written first, then the
    two separator swaps.
    """
    lowered = name.lower()
    out = [lowered]
    for swapped in (lowered.replace("-", "_"), lowered.replace("_", "-")):
        if swapped not in out:
            out.append(swapped)
    return out


def latest_crates_io(name: str) -> str | None:
    """Highest non-yanked, non-prerelease version on crates.io."""
    body = None
    for candidate in _index_name_candidates(name):
        try:
            body = _get(f"https://index.crates.io/{_index_path(candidate)}")
            break
        except urllib.error.HTTPError as e:
            if e.code == 404:
                continue
            raise
    if body is None:
        return None
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


def latest_npm(name: str) -> str | None:
    """The version npm itself resolves `npm install <name>` to.

    The `latest` dist-tag is the registry's own answer, so prereleases are
    excluded without this caller filtering them — npm publishes those under
    `next` / `beta` tags instead. Scoped names carry a `/` that has to be
    encoded, or the registry reads it as a path segment.
    """
    try:
        body = _get(f"https://registry.npmjs.org/{name.replace('/', '%2f')}/latest")
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return None
        raise
    version = json.loads(body).get("version")
    return version if isinstance(version, str) else None


def latest_dart_sdk() -> str | None:
    body = _get("https://storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION")
    return json.loads(body).get("version")


def latest_flutter_sdk() -> str | None:
    body = _get("https://storage.googleapis.com/flutter_infra_release/releases/releases_macos.json")
    doc = json.loads(body)
    stable = doc.get("current_release", {}).get("stable")
    for release in doc.get("releases", []):
        if release.get("hash") == stable:
            return release.get("version")
    return None


def latest_node() -> str | None:
    body = _get("https://nodejs.org/dist/index.json")
    doc = json.loads(body)
    return doc[0]["version"].lstrip("v") if doc else None


# An SDK is not a registry package, and looking one up on a package registry
# is worse than not looking at all: pub.dev really does host unrelated
# packages named `sdk` (0.1.0) and `flutter` (0.0.20), so the obvious
# implementation reports the Dart SDK constraint `^3.9.0` as behind `0.1.0`.
# Measured 2026-09-04. Each SDK gets its own release feed, keyed by the name
# the manifest uses for it (pubspec calls the Dart SDK `sdk`).
SDK_FEEDS = {
    "sdk": latest_dart_sdk,      # pubspec `environment.sdk` IS the Dart SDK
    "dart": latest_dart_sdk,
    "flutter": latest_flutter_sdk,
    "node": latest_node,
}

# One release feed per language. A language absent from this map is reported
# as `unswept`, and never swept against another language's registry. The
# crates.io fallback this replaces split fourteen npm pins two ways, both
# wrong: `next`, `react`, `typescript`, `vite` and `prettier` exist as
# unrelated crates, so those five were compared against a dead crate's version
# and called `behind`; the other nine 404'd, which left the version column `?`
# and the verdict `current` — a clean over pins nothing had compared (#124).
# The second half is the worse one, and it is the same false clean as #105,
# which `read_profile_pins`'s docstring predicted for "an npm list".
REGISTRY_FEEDS = {
    "rust": latest_crates_io,
    "dart": latest_pub_dev,
    "typescript": latest_npm,
}


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
    rather than pattern-matched. Any table carrying a name key and `version`
    is a pin, wherever it sits — including inside the template's own comments,
    which is where the optional ready-to-uncomment blocks live. See
    `commented_toml`; those pins carry a `(commented)` unit label.

    The name key is language-specific and there is no way around that: Rust
    manifests say `crate`, pub manifests say `package` (`pub_reference.schema.yaml`),
    and an npm list would say something else again. The earlier version of
    this function accepted `crate` only while its own docstring claimed "no
    profile-specific key names", so every Dart pin in the tree went unswept
    and the report said all-current over pins it had never read (#105).
    Accept both, and take the registry to look in from the profile's own
    declared `language:` rather than a constant.

    `profile.yaml` is swept too, for SDK constraints — see `read_profile_sdks`.
    """
    pins: list[Pin] = []
    for template in sorted(root.glob("profiles/*/architecture.toml.template")):
        profile = f"profiles/{template.parent.name}"
        language = profile_language(template.parent)
        try:
            doc = tomllib.loads(template.read_text())
        except tomllib.TOMLDecodeError as e:
            print(f"warning: {profile} architecture template does not parse: {e}", file=sys.stderr)
            continue
        for name, version in _walk_dep_refs(doc):
            pins.append(Pin(profile, language, name, version, "registry"))

        # The same template's commented-out optional blocks. These are shipped
        # content — a project author uncomments one and gets whatever it says
        # — but `tomllib` sees a comment, so they were never swept, and two
        # had gone a major version stale before anyone read them by hand
        # (#111). Labelled `(commented)` because the row is about an inert
        # pin: nothing is broken until someone uncomments it.
        hidden = commented_toml(template.read_text())
        if hidden:
            for name, version in _walk_dep_refs(tomllib.loads(hidden)):
                pins.append(Pin(f"{profile} (commented)", language, name, version, "registry"))
    pins.extend(read_profile_sdks(root))
    return pins


def profile_language(profile_dir: Path) -> str:
    """The language a profile declares, defaulting to rust.

    Every shipped profile.yaml carries `language:` at the top level. Reading
    it is what lets the registry lookup follow the profile instead of a
    hardcoded "rust" — the same constant that had a found pub pin being
    queried on crates.io while `latest_pub_dev` sat unreachable in this file.
    """
    manifest = profile_dir / "profile.yaml"
    if not manifest.is_file():
        return "rust"
    doc = yaml.safe_load(manifest.read_text()) or {}
    lang = doc.get("language")
    return lang if isinstance(lang, str) else "rust"


def read_profile_sdks(root: Path) -> list[Pin]:
    """SDK constraints declared in `profile.yaml`, as their own pin kind.

    A profile pins more than packages. `flutter-app/profile.yaml` declares
    `sdk: "^3.9.0"` and `flutter: ">=3.35.0"`, and 38-C's `engines.node` is
    the same shape. These reach every bootstrapped project exactly like a
    crate pin does, and nothing has ever read them.

    They are NOT registry packages, and that distinction is the whole reason
    this is a separate function. pub.dev hosts unrelated packages named `sdk`
    and `flutter`; resolving these against a package registry would report
    the Dart SDK constraint as behind version 0.1.0 of an abandoned package.
    A false finding that names a real-looking version is worse than the false
    clean it replaces. Each SDK resolves against its own release feed, and an
    SDK this function does not have a feed for is reported `unswept` rather
    than dropped — a gap that says so beats a gap that looks like coverage.
    """
    pins: list[Pin] = []
    for manifest in sorted(root.glob("profiles/*/profile.yaml")):
        profile = f"profiles/{manifest.parent.name}"
        doc = yaml.safe_load(manifest.read_text()) or {}
        language = doc.get("language") if isinstance(doc.get("language"), str) else "rust"
        for name, constraint in _walk_sdk_constraints(doc):
            pins.append(Pin(profile, language, name, constraint, "sdk"))
    return pins


# Manifest blocks that hold SDK/toolchain constraints rather than packages:
# pubspec's `environment`, package.json's `engines`.
_SDK_BLOCKS = ("environment", "engines")


def _walk_sdk_constraints(node) -> list[tuple[str, str]]:
    found: list[tuple[str, str]] = []
    if isinstance(node, dict):
        for key, value in node.items():
            if key in _SDK_BLOCKS and isinstance(value, dict):
                for sdk, constraint in value.items():
                    if isinstance(constraint, str):
                        found.append((sdk, constraint))
            else:
                found.extend(_walk_sdk_constraints(value))
    elif isinstance(node, list):
        for value in node:
            found.extend(_walk_sdk_constraints(value))
    return found


def _is_path_source(source) -> bool:
    """Does this `source` point at the local filesystem?

    Two shapes reach here: the string form (`source = "path"`) and the table
    form (`source = { path = "../x" }`). `"workspace"`, `"crates-io"` and a
    git table are all still registry-or-remote and stay swept.
    """
    if isinstance(source, str):
        return source == "path"
    if isinstance(source, dict):
        return "path" in source
    return False


# A commented line is kept only if it looks like TOML: a table header, or a
# `key =` assignment. Prose inside the same comment run ("Optional: enable an
# allocator only when modules need `alloc`.") matches neither and drops out.
_TOML_HEADER = re.compile(r"^\[\[?[A-Za-z_][A-Za-z0-9_.\-\"']*\]\]?$")
_TOML_KEYLINE = re.compile(r"^[A-Za-z_][A-Za-z0-9_.-]*\s*=")


def commented_toml(text: str) -> str:
    """The TOML hiding inside a template's comments.

    Profile templates offer optional dependency blocks as ready-to-uncomment
    comments. `tomllib` sees a comment, so those pins were invisible to this
    sweep for its whole life, and two of them had gone a major version stale
    before anyone read them by hand (#111).

    Uncommenting is not enough on its own. Prose sits in the same comment run
    as the pin it introduces, and some prose looks like a key —
    `source = "workspace"   — <dep>.workspace = true inheritance` passes any
    reasonable line filter and then fails the parse, which would throw away
    the real pins beside it. So lines are added one at a time and a line that
    breaks the document is dropped. Slow in principle, invisible in practice:
    the templates are a few dozen comment lines each.

    Returns a TOML document, possibly empty. The caller parses it.
    """
    kept: list[str] = []
    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped.startswith("#"):
            continue
        body = stripped[1:]
        body = body[1:] if body.startswith(" ") else body
        body = body.strip()
        if not (_TOML_HEADER.match(body) or _TOML_KEYLINE.match(body)):
            continue
        candidate = kept + [body]
        try:
            tomllib.loads("\n".join(candidate))
        except tomllib.TOMLDecodeError:
            continue
        kept = candidate
    return "\n".join(kept)


def _walk_dep_refs(node) -> list[tuple[str, str]]:
    """Every `<name key> + version` pair in a parsed manifest.

    `crate` is Rust's name key, `package` is pub's. A table carrying a source
    key but no `version` (a path, git or sdk dep) is not a pin and is skipped
    by the `version` requirement alone — which is also what keeps pubspec's
    `{ package = "flutter", sdk = "flutter" }` out of the registry pins.

    A table that carries BOTH a `version` and a path source is skipped too.
    No registry has ever heard of it: the commented examples this function
    now reads are full of placeholders like
    `{ crate = "my-core", version = "0.1.0", source = { path = "crates/my-core" } }`,
    and looking those up produces an `unswept` row about a name that was
    never meant to resolve.
    """
    found: list[tuple[str, str]] = []
    if isinstance(node, dict):
        for name_key in ("crate", "package"):
            if isinstance(node.get(name_key), str) and isinstance(node.get("version"), str):
                if not _is_path_source(node.get("source")):
                    found.append((node[name_key], node["version"]))
                break
        for value in node.values():
            found.extend(_walk_dep_refs(value))
    elif isinstance(node, list):
        for value in node:
            found.extend(_walk_dep_refs(value))
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
    # `>=X` is an open lower bound: anything at or above it satisfies it, so
    # it can never be "behind". Strip-and-caret-compare would misread it —
    # `_components(">=3.35.0")` yields (0, 35, 0), because `int(">=3")` fails.
    if req.lstrip().startswith(">"):
        return _components(version) >= _components(req.lstrip("> ="))
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


def ahead_of(req: str, version: str) -> bool:
    """Is the pin ABOVE the newest thing the registry offered?

    `satisfies` asks whether the candidate sits inside the declared range, and
    a candidate *below* the floor is outside it — so without asking this first,
    a pin the world has not caught up with reads as `behind`, gets built
    against the older version, and fails. `blocks/itm` pins `itm 0.9.0-rc.1`
    while the newest stable is `0.3.1`: the pin is the newest version
    published, and calling it behind sent the weekly job red on a correct pin
    (#112). Being ahead is never a finding — there is nothing to bump to.

    Prerelease identifiers are not compared, because `_components` drops them
    and the stable feeds never return one. That is why the row says so.
    """
    return _components(version) < _components(req.lstrip("^~>= "))


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
    # Open lower bounds. SDK constraints use these (`flutter: ">=3.35.0"`),
    # and the caret path misreads them: `_components(">=3.35.0")` is
    # (0, 35, 0), because int(">=3") fails and falls back to 0 — which made
    # a satisfied constraint report as behind.
    (">=3.35.0", "3.47.2", True, ">= is an open upper end; a major bump still satisfies it"),
    (">=3.35.0", "3.35.0", True, ">= includes its own floor"),
    (">=3.35.0", "3.34.9", False, "below the floor of a >= constraint"),
    (">= 3.35.0", "3.47.2", True, ">= tolerates a space before the version"),
]

# `satisfies` says "the candidate is outside the declared range" for BOTH a
# candidate that is too new and one that is too old. Only the second is
# `ahead`, and telling them apart is what keeps a correct pin out of the red
# (#112). These cases exist because the `satisfies` table above deliberately
# asserts False for ("1.0.3", "1.0.2") — that answer is right, and the bug was
# the caller reading it as "behind".
AHEAD_CASES = [
    ("0.9.0-rc.1", "0.3.1", True, "a prerelease pin the stable line has not reached — blocks/itm"),
    ("0.36", "0.37.0", False, "genuinely behind: the feed is newer"),
    ("0.36", "0.36.1", False, "a patch above the pin is not ahead"),
    ("1.0.3", "1.0.2", True, "the feed's newest is below the pinned floor"),
    ("^19.2.0", "19.3.0", False, "caret pin, feed newer within the major"),
    ("2.0.0", "2.0.0", False, "equal is not ahead"),
    (">=3.35.0", "3.34.9", True, "the feed has not reached our declared minimum"),
]


# `commented_toml` has to survive the shapes that actually ship: prose in the
# same comment run as the pin it introduces, prose that looks like a key, and
# path placeholders that no registry has heard of. Each case is
# (label, template text, expected [(name, version)]).
COMMENTED_CASES = [
    (
        "prose above a pin does not take the pin with it",
        '# Optional: enable an allocator only when modules need `alloc`.\n'
        '# [runtime.allocator]\n# crate = "embedded-alloc"\n# version = "^0.7"\n',
        [("embedded-alloc", "^0.7")],
    ),
    (
        "prose that looks like a key is dropped, the pin beside it survives",
        '# [[external_deps]]\n# crate = "rtrb"\n# version = "^0.4"\n'
        '#   source = "workspace"   — `<dep>.workspace = true` inheritance\n',
        [("rtrb", "^0.4")],
    ),
    (
        "a path placeholder is not a registry pin",
        '# [[workspace_dependencies]]\n# crate = "my-core"\n# version = "0.1.0"\n'
        '# source = { path = "crates/my-core" }\n',
        [],
    ),
    (
        "uncommented lines are not this pass's business",
        '[logger.provider]\ncrate = "defmt"\nversion = "^1"\n',
        [],
    ),
    (
        "a bare version with no name key is not a pin",
        '# [package]\n# version = "0.1.0"\n',
        [],
    ),
]


def self_test() -> int:
    bad = 0
    for req, ver, want, why in SATISFIES_CASES:
        got = satisfies(req, ver)
        if got != want:
            bad += 1
        print(f"{'ok  ' if got == want else 'FAIL'} satisfies({req!r}, {ver!r}) = {got} — {why}")
    print()
    for req, ver, want, why in AHEAD_CASES:
        got = ahead_of(req, ver)
        if got != want:
            bad += 1
        print(f"{'ok  ' if got == want else 'FAIL'} ahead_of({req!r}, {ver!r}) = {got} — {why}")
    print()
    for label, text, want in COMMENTED_CASES:
        got = _walk_dep_refs(tomllib.loads(commented_toml(text)))
        ok = got == want
        if not ok:
            bad += 1
        print(f"{'ok  ' if ok else 'FAIL'} commented_toml -> {got} — {label}")
    total = len(SATISFIES_CASES) + len(AHEAD_CASES) + len(COMMENTED_CASES)
    print(f"\n{total - bad}/{total} passed")
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
        elif pin.kind == "sdk":
            feed = SDK_FEEDS.get(pin.name)
            pinned_short = pin.pinned
            if feed is None:
                # No feed for this SDK. Say so; do not fall through to a
                # package registry, and do not drop it silently. `unswept` and
                # not `behind`: nothing was compared, so claiming a direction
                # would be inventing one.
                rows.append((pin, pinned_short, "?", "unswept", "no release feed for this SDK"))
                continue
            current = feed() or "?"
            behind = current != "?" and not satisfies(pin.pinned, current)
        else:
            lookup = REGISTRY_FEEDS.get(pin.language)
            pinned_short = pin.pinned
            if lookup is None:
                rows.append(
                    (pin, pinned_short, "?", "unswept",
                     f"no release feed for language `{pin.language}`")
                )
                continue
            current = lookup(pin.name) or "?"
            behind = current != "?" and not satisfies(pin.pinned, current)

        # A comparison that did not happen must never render as `current`.
        # A 404, an unreachable git remote, or a feed that answered with
        # nothing all leave `current` as "?", which makes `behind` False and
        # used to print a clean row. Nine npm pins were reported current that
        # way, having been compared to nothing at all (#124) — the same false
        # clean as #105, and the reason `unswept` is a state rather than a
        # note on some other state.
        if current == "?":
            rows.append(
                (pin, pinned_short, current, "unswept",
                 "the feed returned nothing for this name")
            )
            continue

        # The registry's newest is OLDER than what we pin. Ask before `behind`,
        # or a pin nothing has caught up with is downgraded and built against
        # the older version (#112).
        # `pin.kind != "git"`: a commit SHA has no version ordering. `_components`
        # would read one as (0, 0, 0) — or worse, parse an all-digit SHA into a
        # number — and compare two meaningless tuples.
        if behind and current != "?" and pin.kind != "git" and ahead_of(pin.pinned, current):
            rows.append(
                (pin, pinned_short, current, "ahead",
                 "pin is newer than the feed's newest; prereleases not compared")
            )
            continue

        if not behind:
            rows.append((pin, pinned_short, current, "current", ""))
            continue
        if policy:
            rows.append((pin, pinned_short, current, "held", policy.get("reason", "")))
            continue

        if args.no_build or pin.language != "rust" or pin.kind == "sdk":
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
        marker = {
            "current": "  ", "held": "· ", "ahead": "↑ ",
            "unswept": "? ", "behind": "→ ", "BREAKS": "✗ ",
        }[state]
        line = f"{marker}{pin.unit:<{width}}  {pin.name:<26} {pinned:<14} {current:<14} {state}"
        print(line + (f"   ({note})" if note else ""))

    print()
    states = ("current", "held", "ahead", "unswept", "behind", "BREAKS")
    counts = {s: sum(1 for *_, st, _ in rows if st == s) for s in states}
    print(
        f"current: {counts['current']}   held: {counts['held']}   "
        f"ahead: {counts['ahead']}   unswept: {counts['unswept']}   "
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
        if st == "unswept"
        or (st == "behind" and ("not checked" in note or "not built" in note))
    ]
    if unverified:
        print(
            f"\nUNVERIFIED — {len(unverified)} pin(s) with nothing able to say whether they break:"
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
