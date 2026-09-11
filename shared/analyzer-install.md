# Installing `specdev-analyzer`

Use this when `specdev-analyzer` is not on PATH, or when it must be
re-installed because it is older than the plugin (`MANIFEST-002`, or a
`MANIFEST-001` naming a file under the plugin tree).

**The plugin and the analyzer are one unit and must come from the same
GitHub organization.** The analyzer embeds the JSON Schemas it validates the
plugin's files against, so a mismatched pair validates the user's project
against schemas the plugin does not ship.

## Step 1 — find the org, do not guess it

Both repositories are **private**, so the install URL depends on which
organization the user was granted access to. Read it off the plugin's own
remote rather than assuming:

```sh
git -C "${CLAUDE_PLUGIN_ROOT}" remote get-url origin
```

The org is the path segment after the host:

```
git@github.com:cosmikwolf/specdev.git      ->  cosmikwolf
https://github.com/kink-engineering/specdev.git  ->  kink-engineering
```

**If that command exits non-zero** (128 when the plugin tree is not a git
checkout — a marketplace install need not keep `.git`), do **not** invent an
org. Ask the user which organization they have access to, and offer the
local-checkout install below, which needs no org at all.

## Step 2 — tell the user one of these

From a local checkout — no network and no org access needed, and the
supported path for anyone developing against the pair:

```sh
cargo install --path <analyzer-checkout>
```

From the private repository, with the org from Step 1:

```sh
cargo install --git https://github.com/<org>/specdev-analyzer
```

Add `--force` when re-installing; an install at an unchanged version still
replaces the binary. Requires a stable Rust toolchain, at least **1.86**.

## Why the README says `<org>` and this file does not

`README.md` and `scripts/resolve-context` print `<org>` as a literal
placeholder because a **human** reads them and substitutes it. This file is
read by **Claude**, which can run a command — so it derives the value instead
of asking for it. Both are correct for their reader; do not "unify" them.

Note also that `${{ github.repository_owner }}` works only in GitHub Actions
workflow YAML. It renders literally in Markdown and must never appear here.

## Then stop

Do not attempt the skill's own work without the analyzer. Report the install
command and stop; retry once the user confirms the install.
