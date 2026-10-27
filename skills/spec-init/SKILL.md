---
name: spec-init
description: This skill should be used when the user asks to "init specdev", "set up a new spec project", "scaffold a spec-driven project", "bootstrap a profile", "create spec.toml", "initialize specs/", or starts a new specdev project from scratch. Wraps `specdev-analyzer bootstrap`, picks a profile from whatever ships under `${CLAUDE_PLUGIN_ROOT}/profiles/` (including end-user-authored profiles), and seeds the `specs/specdev/` tree. Use for empty or greenfield projects. Pass `--non-interactive --profile <name> --language <lang> --name <project> [<target>]` to skip the interview (for automation or fluent users who already know the answers). To bring an existing codebase in (Rust, TypeScript, Dart or Python), run /spec-init first, then /spec-reverse to author its modules from the code.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: "[--non-interactive --profile <name> --language <lang> --name <project> [<target-dir>]]"
---

# /spec-init — Project scaffolding

Initialize a new spec-driven project by interviewing the user, then delegating file emission to `specdev-analyzer bootstrap`. The analyzer is the source of truth for what a freshly-bootstrapped project looks like; this skill is a thin wrapper that picks the profile, fills the architecture's required fields, and verifies the result.

The current model is **profiles + blocks + bridges**: profiles own project shape; blocks contribute concrete frameworks/libraries/hardware; bridges wire pairs of blocks.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry. Do not attempt to write files without the analyzer.

---

## Step 0 — session env preamble

Run the export from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md` before any analyzer invocation:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

See the shared file for the full rationale and skip-conditions. Short version: `CLAUDE_PLUGIN_ROOT` is set in this skill body but is **not** inherited by Bash subshells, so exporting `SPECDEV_PLUGIN_ROOT` once makes plugin-root resolution work for the rest of the session.

## Step 0.5 — resolve mode

Inspect `$ARGUMENTS`:

- If it carries `--non-interactive` AND all of `--profile <name>`, `--language <lang>`, `--name <project>` (target dir is positional and optional, default `.`), this is **non-interactive mode**. Skip Steps 2, 3, 4 and 6 entirely; carry the provided values into Step 5 verbatim. Step 1 (refuse-to-clobber) and Steps 7–8 still run.
- Otherwise this is **interactive mode** — continue with Step 1 below. Never silently fill in missing flags from defaults; interactive mode's whole point is surfacing defaults via Step 2's manifest detection and Step 4's interview.

If `--non-interactive` is passed without all three required flags, stop and ask the user for the missing ones in a single message.

## Step 1 — refuse to clobber

If `<project_root>/specs/specdev/` already exists, the project is already bootstrapped. Read `spec.toml` and show the current `[project]` block. Ask:

- **Keep** — leave it untouched, stop here.
- **Update one field** — point the user at a specific file (`spec.toml` for name/license/description; `architecture.toml` for `[platform]`/`[runtime]`/etc.). This skill does not edit existing projects.
- **Re-bootstrap into a fresh dir** — confirm a new path, treat that as the target.

If `specs/specdev/` is absent, continue — bootstrapping it is the whole point of this skill.

## Step 2 — detect existing build metadata

Scan the project root for build manifests and extract fields per `references/build-manifest-detection.md`. Use extracted values as defaults for the Step 4 interview — show every defaulted value and let the user confirm or change it. This step is read-only; nothing is written until Step 5.

## Step 3 — list available profiles

Profiles ship in the plugin tree. List them dynamically — never hardcode the set, since end users may author their own profiles:

```sh
ls "${CLAUDE_PLUGIN_ROOT}/profiles/"
```

For each profile name returned, read `${CLAUDE_PLUGIN_ROOT}/profiles/<name>/README.md` and surface one line per profile (the README's first paragraph or a one-sentence summary). Present the list to the user verbatim — every directory under `profiles/` is a valid choice; user-authored profiles surface here the same way as plugin-shipped ones.

## Step 4 — interview

Ask all of these in one message. Show defaults from Step 2 in parentheses:

1. **Project name** — lowercase, digits, underscores, hyphens; must start with a letter; ≤64 chars. (default: detected `[package].name` or directory name)
2. **Profile** — one of the names listed in Step 3. Default heuristics for shipped profiles, in this order (first match wins): `rust-workspace` if `[workspace]` was detected; `firmware-layered` if a `Cargo.toml` with `cortex-m`/`embassy`/`stm32*`/`nrf*`/`esp*` deps was detected; `audio-plugin` if a `nih-plug` / `vst3-sys` / `clack-host` dep was detected; `rust-app` if a `src/main.rs`, a `[[bin]]` section, or a `src/bin/` directory was detected (a standalone executable that is none of the above); `library-api` if a `Cargo.toml` with no binary target was detected; `nextjs-app` if a `package.json` with a `next` dependency was detected; `flutter-app` if a `pubspec.yaml` with a `flutter` SDK dependency was detected. Otherwise ask.

When the choice is between `rust-app` and `library-api` and the detection is ambiguous, ask rather than guessing — the distinguishing question is "is this crate consumed by other crates, or does it run on its own?", and it decides whether the API surface or the layering is the load-bearing concern. For user-authored profiles no default is inferred — ask.
3. **Language** — `rust`, `dart`, `typescript` or `python`. The profile decides: each profile's `profile.yaml` declares its `language:`, so offer the language matching the chosen profile (`dart` for `flutter-app` and `dart-package`, `typescript` for `nextjs-app` and `ts-library`, `python` for `python-library` and `python-ffi`, `rust` for the Rust profiles); a mismatch fails at bootstrap. `c` and `cpp` are in the schema enum and have an audit tier, but no profile targets them yet. (default: the chosen profile's declared language)
4. **Bootstrap directory** — usually the current working directory, but the analyzer creates it if absent and refuses if `specs/specdev/` is already present. (default: `.`)

**When the profile lists `toolchains:`** (`python-library`, `python-ffi`, `ts-library`), also ask **which tool** the project uses — one of the profile's `toolchains` keys (Python: `uv`, `poetry`, `pip`, default `uv`; `ts-library`: `pnpm`, `npm`, default `pnpm`). After bootstrap, write the answer to `spec.toml` `[project] toolchain` (the template seeds the default) and run `specdev-analyzer gate --emit`. For the Python profiles, also set `[paths].source` to the import package's directory: the template writes `src/<project name>`, which is wrong when the name has a `-` (the import name uses `_`). For `python-ffi`, `specs/architecture.toml` `[native] module` has the same rule, and the Rust crate is a second project: bootstrap it with `library-api` in the directory `[tool.maturin] manifest-path` will name, then `/spec-inline pyo3` there.

These four questions are all the schema requires. Profile-specific facts (the target triple, connections) live in `architecture.toml` and are filled in Step 6, after the analyzer has emitted templates. Versions, licenses and dependencies stay in the build manifest.

## Step 5 — run bootstrap

`<bootstrap_dir>` is a required trailing positional and the analyzer creates it if it does not exist, so run this from wherever you are and give the target as a path — there is no project root to `cd` into yet when the directory is new:

```sh
specdev-analyzer bootstrap \
  --profile <profile> \
  --language <language> \
  --name <project_name> \
  <bootstrap_dir>
```

This emits the full `specs/specdev/` machinery (schemas, reductions, `domain.toml`, `profile.yaml`, and the profile's implementation guidance in `specs/specdev/guidance/profile.md`), plus `spec.toml`, the empty `specs/modules/` directory, the gate script `scripts/specdev-gates.sh`, and the profile's paired files (config files written once when absent — for example the CI workflow template). Templates carry profile-specific commented hints.

If the analyzer fails:
- `BOOTSTRAP-001` — `specs/specdev/` already exists, or the profile name is invalid; surface message verbatim.
- `BOOTSTRAP-003` — a file already sits where bootstrap planned to write (`spec.toml`, `specs/architecture.toml`, the CI workflow, …). The message names every collision and **nothing was written**. This is the brownfield case: the project already has spec files, hand-authored or copied from a sibling project. Surface it verbatim and ask the user before going further — `--force` overwrites them, and their contents are not recoverable afterwards. Moving the files aside and re-running is usually the better answer. Note this does *not* fire for an existing `src/` or build manifest; bootstrap never writes either.
- `BOOTSTRAP-002` — unknown placeholder in profile template (a plugin authoring bug); ask the user to file an issue against the plugin.
- `PLUGIN-001` — plugin root not resolvable. Run Step 0's `export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"` if you skipped it; otherwise pass `--plugin-root <path-to-specdev-plugin>` explicitly. Do not rely on `CLAUDE_PLUGIN_ROOT` propagating into Bash subshells.

Do not attempt to repair these errors by hand-writing files.

## Step 5.5 — the compile gates and CI the project just inherited

Bootstrap copies the profile's `profile.yaml` byte-faithfully, so the new `specs/specdev/profile.yaml` carries the profile's **`gates:`** block — the compile gates as data, one `label` + `steps` (argv lists) per gate — and renders them into `scripts/specdev-gates.sh`. Read the block back and show the user what the project inherited:

```sh
awk '/^gates:/{f=1;print;next} f&&/^[^[:space:]]/{exit} f' specs/specdev/profile.yaml
```

Three facts the user needs now:

- **The script is what CI runs.** `sh scripts/specdev-gates.sh` runs `specdev-analyzer check` first and then every gate, stopping at the first failure. The profile's CI workflow template (when it ships one) installs the toolchain the profile README lists, builds the analyzer, and runs the script. A missing tool stops the script with exit 2 and a line naming the command.
- **The block is the project's now.** Editing a step, adding a gate or changing a label is a normal edit to `specs/specdev/profile.yaml`; then `specdev-analyzer gate --emit` rewrites the script (`validate` reports a script that differs from the block).
- **Commit the lockfile.** CI installs are frozen: a CI run refuses to change the lockfile.

If the chosen profile is **user-authored and ships no `gates:` block**, say so here and give the shape to write:

```yaml
gates:
  - label: fast
    steps:
      - [<program>, <arg>, <arg>]
```

`steps` is a list of **argv lists**, never a shell string: one exit code per step, and no second quoting grammar. A `${root.<table>.<field>}` token in an argument resolves against `specs/architecture.toml` when the script is rendered. `/spec-create-profile` § "Step 1 — interview the project shape" is the authoring reference.

## Step 6 — confirm or fill architecture defaults

Read the emitted `specs/architecture.toml`. The template ships pre-filled with sane defaults; bootstrap-emitted architecture is valid as-is. This step is interview-driven refinement, not error recovery — only edit fields the user actually wants to change.

For per-profile refinement shapes, see `references/architecture-refinement.md`. For user-authored profiles, read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md` and the emitted `specs/architecture.toml` together — the README enumerates which fields the profile considers required vs optional.

## Step 7 — verify

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Expect a clean report — bootstrap emits valid templates, and Step 6's optional refinements stay within schema. If `SCHEMA-ARCH` or `SCHEMA` errors appear, the user's Step 6 edits violated a field constraint; fix the offending field based on the message — do not delete fields to silence checks. Re-run until clean.

## Step 8 — report

Run `/spec-config` first. It writes `.claude/rules/specdev.md` and `specdev-feedback.md` at the
project root, each only when that file is absent, and the project owns both from then on. On a
fresh bootstrap it creates both. It needs no analyzer.

Summarize:
- Files created (one line each, with paths).
- What `/spec-config` reported for each of its two files: created, or already present.
- The selected profile and a one-line reminder of what it shapes (firmware-layered → layered embedded; library-api → Rust library; rust-app → standalone Rust application, testable core plus thin entry point; audio-plugin → VST3/CLAP/standalone audio plugin; rust-workspace → workspace root that walks per-member crates; flutter-app → Flutter application; nextjs-app → Next.js App Router application). For a user-authored profile, quote its README's first line.
- The gate registry the project inherited (Step 5.5): the gate labels and the script path, or the shape to write when the profile ships none.
- Validate result.
- **Suggested next steps**: render per-profile guidance from `${CLAUDE_PLUGIN_ROOT}/shared/profile-next-steps.md` (it has a section per shipped profile plus a fallback for user-authored ones).

## What this skill does NOT do

- **Author module specs.** That's `/spec-design`'s job. Bootstrap leaves `specs/modules/` empty.
- **Inline blocks or bridges.** Use `/spec-inline <block>` and `/spec-inline-bridge <a>+<b>` after bootstrap.
- **Edit an existing project.** Re-running `spec-init` over a bootstrapped tree is refused at Step 1; for changes, edit the relevant template-emitted file directly.
- **Pick MCU hardware.** It asks the user; it does not auto-detect from `.cargo/config.toml` target triples (the architecture template defaults are sane for the most common case but require confirmation).

## Additional resources

- `references/build-manifest-detection.md` — manifests scanned in Step 2 and fields extracted as interview defaults.
- `references/architecture-refinement.md` — illustrative per-profile architecture refinements for Step 6 (the firmware-layered target triple; connections and project decisions).
- `references/requirements-readme-template.md` — verbatim body for an optional `specs/requirements/README.md` file (write only if the project intends to track requirements).
- `${CLAUDE_PLUGIN_ROOT}/shared/profile-next-steps.md` — per-profile suggested next steps rendered in Step 8.
