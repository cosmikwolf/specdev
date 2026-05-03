---
name: spec-bootstrap
description: This skill should be used when the user asks to "bootstrap a project", "run specdev bootstrap", "scaffold from a profile non-interactively", or wants to invoke `specdev-analyzer bootstrap` directly with explicit profile/language/name arguments. Thin wrapper over the analyzer's `bootstrap` subcommand. For interview-driven scaffolding (with build-manifest detection and architecture refinement), use `/spec-init` instead.
allowed-tools: Read, Bash, Glob, Grep
argument-hint: --profile <name> --language <lang> --name <project> [<target-dir>]
---

# /spec-bootstrap — Thin wrapper over `specdev-analyzer bootstrap`

Run the analyzer's `bootstrap` subcommand and surface its output verbatim. Do not interview the user — `/spec-init` is the interview-driven scaffolding skill. This skill is for callers who already know the four required inputs (profile, language, name, target dir).

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — parse arguments

User input: `$ARGUMENTS`. Required flags:

- `--profile <name>` — must match a directory under `${CLAUDE_PLUGIN_ROOT}/profiles/`. v1 ships `firmware-layered`, `library-api`, `rust-workspace`. List them with `ls "${CLAUDE_PLUGIN_ROOT}/profiles/"` if the user asks.
- `--language <lang>` — `rust` is the only v1 option. C lands in Phase 9.
- `--name <project>` — lowercase letters/digits/underscores/hyphens; must start with a letter; ≤64 chars. Becomes `[project].name` in the generated `spec.toml`.

Optional:

- `<target>` (positional) — directory to bootstrap into. Defaults to `.` (cwd). Created if absent; **fails** if `<target>/specs/specdev/` is already present.

If any required flag is missing, ask the user once for all of them in a single message — do not guess. If the target tree already has `specs/specdev/`, stop and tell the user to use `/spec-init`'s "re-bootstrap into a fresh dir" path or pick a different target.

## Step 2 — run bootstrap

```sh
specdev-analyzer bootstrap \
  --profile <profile> \
  --language <language> \
  --name <project_name> \
  <target>
```

Capture stdout, stderr, and exit code.

Exit codes:
- `0` — clean. Go to Step 3.
- non-zero — surface stderr verbatim. Do not paraphrase. Common error families:
  - `BOOTSTRAP-001` — target tree already exists or profile name invalid. Show the message; suggest a different target dir.
  - `BOOTSTRAP-002` — unknown placeholder in profile template. Plugin-author bug; ask the user to file an issue against the plugin tree.
  - `PLUGIN-001` — plugin root not resolvable. Re-run the Step 0 preamble (`export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"`) or pass `--plugin-root <path>` explicitly. See `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`.
  - `LAYOUT-001..006` — layout-loader errors; show verbatim.

Do not attempt to repair these by hand-writing files.

## Step 3 — validate and report

After a successful bootstrap, run:

```sh
specdev-analyzer validate --no-color
```

from the target directory. A freshly bootstrapped project should validate clean. If it doesn't, surface the findings — they indicate a plugin-author issue, not user error.

Report:

- Profile, language, name, and target directory used.
- Number of files emitted (one-line summary of the new tree, e.g., `specs/specdev/{schemas,reductions,codegen}/...`).
- Validate result.
- Suggested next step:
  - `firmware-layered` → `/spec-inline stm32h5-rs && /spec-inline embassy && /spec-inline-bridge embassy+stm32h5-rs`, then `/spec-design <module>`.
  - `library-api` → `/spec-design <module>`.
  - `rust-workspace` → bootstrap each member with `cd <member> && /spec-bootstrap --profile library-api --language rust --name <member-name>`.

## What this skill does NOT do

- **Interview the user.** Use `/spec-init` if the user wants build-manifest detection, profile suggestions, and architecture refinement.
- **Edit `architecture.toml` defaults.** The bootstrap-emitted templates are valid as-is; refinement is the user's call (or `/spec-init`'s).
- **Inline blocks or bridges.** That's `/spec-inline` / `/spec-inline-bridge`.
- **Re-bootstrap an existing project.** Refused at Step 1 by checking for `specs/specdev/`.
