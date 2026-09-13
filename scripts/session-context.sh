#!/bin/sh
# session-context.sh — SessionStart hook: state the project fact, and nothing else.
#
# Walks up from the current directory looking for a project root: a directory
# that holds BOTH `spec.toml` and `specs/specdev/`. Both are required. A bare
# `spec.toml` is a generic filename and says nothing about specdev.
#
# On a hit it prints `shared/project-rules.md` on stdout, which Claude Code adds
# to the session as context. On a miss it prints nothing.
#
# EXCEPT when the project has its own copy at `.claude/rules/specdev.md`.
# /spec-config writes that file (and /spec-init always calls /spec-config), so
# on a normally-bootstrapped project it is always there — and Claude Code loads
# everything under `.claude/rules/` into the session already. Printing here too
# put the same ~1.8KB in context twice, every session, in every project. Worse
# than the waste: /spec-config never updates its copy after the first write, so
# the next plugin release that edits project-rules.md would have made the two
# texts diverge, and a session would carry two conflicting rule sets at once.
# The project's copy wins, because it is the one that also works with hooks
# disabled — which is the reason /spec-config writes it at all.
#
# Deliberate constraints. Do not relax one without revisiting the decision that
# authorized this hook:
#   - reads no project file beyond the three existence tests above
#   - prints nothing that depends on project content (existence, never bytes:
#     this script must never compare the two copies or report on staleness)
#   - never invokes specdev-analyzer
#   - writes no file
#   - always exits 0, so a fault here can never block a session

if [ -n "${CLAUDE_PLUGIN_ROOT:-}" ]; then
    PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
else
    SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd)" || exit 0
    PLUGIN_ROOT="$(cd "${SCRIPT_DIR}/.." 2>/dev/null && pwd)" || exit 0
fi

RULES="${PLUGIN_ROOT}/shared/project-rules.md"
[ -f "${RULES}" ] || exit 0

dir="$(pwd)"
while :; do
    if [ -f "${dir}/spec.toml" ] && [ -d "${dir}/specs/specdev" ]; then
        # The project owns a copy; Claude Code already loaded it. Stay quiet.
        if [ -f "${dir}/.claude/rules/specdev.md" ]; then
            exit 0
        fi
        cat "${RULES}"
        exit 0
    fi
    parent="$(dirname "${dir}")"
    [ "${parent}" = "${dir}" ] && break
    dir="${parent}"
done

exit 0
