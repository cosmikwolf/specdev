# Installing `specdev-analyzer`

Use this when `specdev-analyzer` is not on PATH, or when it must be
re-installed because it is older than the plugin (`ANALYZER-VERSION-001` on
stderr, `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin
tree).

`ANALYZER-VERSION-001` is a warning: the command still ran. The analyzer
prints it when its own version is older than `shared/min-analyzer-version`
in this plugin tree. Tell the user, and offer the install below; do not
stop the skill for it (see the end of this file).

**The plugin and the analyzer are one unit and must come from the same
GitHub organization.** The analyzer embeds the JSON Schemas it validates the
plugin's files against, so a mismatched pair validates the user's project
against schemas the plugin does not ship.

## Step 1 — find the org, do not guess it

The canonical repositories are public under `cosmikwolf`, but a fork of the
plugin pairs with the analyzer from the same fork. Read the org off the
plugin's own remote rather than assuming:

```sh
git -C "${CLAUDE_PLUGIN_ROOT}" remote get-url origin
```

The org is the path segment after the host:

```
git@github.com:cosmikwolf/specdev.git      ->  cosmikwolf
https://github.com/<org>/specdev.git      ->  <org>
```

**If that command exits non-zero** (128 when the plugin tree is not a git
checkout — a marketplace install need not keep `.git`), do **not** invent an
org. Use `cosmikwolf`, the canonical org, unless the user says they installed
from a fork; the local-checkout install below needs no org at all.

## Step 2 — tell the user one of these

From a local checkout — no network needed, and the
supported path for anyone developing against the pair:

```sh
cargo install --path <analyzer-checkout>
```

From GitHub, with the org from Step 1:

```sh
cargo install --git https://github.com/<org>/specdev-analyzer --tag v0.6.0
```

Add `--force` when re-installing; an install at an unchanged version still
replaces the binary. Requires a stable Rust toolchain, at least **1.86**.

A prebuilt binary, faster and with no Rust toolchain (from `cosmikwolf` only:
its Releases carry binaries for macOS arm64 and x86_64, Linux x86_64 (musl)
and Windows x86_64). Use the tag the `cargo install` line above names:

```sh
gh release download <tag> -R cosmikwolf/specdev-analyzer -p 'specdev-analyzer-<target>-<tag>.tgz'
tar xzf specdev-analyzer-<target>-<tag>.tgz && mv specdev-analyzer ~/.cargo/bin/
```

`<target>` is the Rust target triple (`rustc -vV` prints it as `host`). When the
repository is public, `cargo binstall --git https://github.com/cosmikwolf/specdev-analyzer
specdev-analyzer` does the same in one step; while it is private, `cargo binstall`
cannot download Release files (measured), so use `gh`.

## Why the README names `cosmikwolf` and this file derives the org

`README.md` and `scripts/resolve-context` name the canonical org because a
**human** reads them and most installs come from there. This file is read by
**Claude**, which can run a command — so it also finds a fork's org. Both are
correct for their reader; do not "unify" them.

Note also that `${{ github.repository_owner }}` works only in GitHub Actions
workflow YAML. It renders literally in Markdown and must never appear here.

## Then stop

When the analyzer is missing, or a `MANIFEST-002` / `MANIFEST-001` names the
plugin tree, do not attempt the skill's own work. Report the install command
and stop; retry once the user confirms the install.

`ANALYZER-VERSION-001` is the one exception: the command ran. Report the
install command once and continue.
