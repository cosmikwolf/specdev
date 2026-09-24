# Plugin-root session preamble

Run this once, before any `specdev-analyzer` invocation in the session:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

If `CLAUDE_PLUGIN_ROOT` is empty (Cursor sets it for plugin hooks only, not in the agent's shell), set `SPECDEV_PLUGIN_ROOT` to the directory above `skills/` in the path of the `SKILL.md` you are running, check that `.claude-plugin/plugin.json` exists there, and use that path wherever these skills write `${CLAUDE_PLUGIN_ROOT}`.

## Why

`CLAUDE_PLUGIN_ROOT` is auto-set inside skill execution contexts but is **not** inherited by the Bash subshells the analyzer is typically invoked from on later turns. Without this export, the analyzer hits `[PLUGIN-001] plugin root not resolvable` on its first run, and every subsequent invocation repeats the failure until the user pieces together the env-propagation gap. Capturing `CLAUDE_PLUGIN_ROOT` into `SPECDEV_PLUGIN_ROOT` once makes plugin-root resolution work for the rest of the session.

## When to skip

Skip the export only if:

- `SPECDEV_PLUGIN_ROOT` is already set in the environment (the user has it in their shell rc), OR
- every analyzer invocation passes `--plugin-root <path>` explicitly, OR
- the project's `spec.toml` declares `[paths].plugin_root` (precedes env vars).

## On `[PLUGIN-001]` errors

If `[PLUGIN-001]` fires:

1. If the export was skipped, run it now and retry.
2. If the export was set but resolution still fails, the value points at the wrong directory — verify `${SPECDEV_PLUGIN_ROOT}/.claude-plugin/plugin.json` exists. If not, ask the user for the correct plugin-tree path and pass `--plugin-root <path>` explicitly.

Do not work around `[PLUGIN-001]` by hand-writing files the analyzer would have produced.

## On `[ANALYZER-VERSION-001]` warnings

Once the export above is in place, every analyzer command compares its own version with `shared/min-analyzer-version` in this plugin tree. When the binary is older, it prints one `warning: [ANALYZER-VERSION-001]` line on stderr. The command still ran, and its exit code and stdout are unchanged.

Tell the user **once per session**: the installed analyzer is older than this plugin needs, and some commands can behave as the skills do not expect. Give the re-install command from `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md`. Then continue the skill. Do not repeat the warning for each later command. Do not read it as the command's failure.
