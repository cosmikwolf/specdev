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
# Deliberate constraints. Do not relax one without revisiting the decision that
# authorized this hook:
#   - reads no project file beyond the two existence tests above
#   - prints nothing that depends on project content
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
        cat "${RULES}"
        exit 0
    fi
    parent="$(dirname "${dir}")"
    [ "${parent}" = "${dir}" ] && break
    dir="${parent}"
done

exit 0
