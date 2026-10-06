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
# At a project root it also relays the analyzer's version warning:
# `specdev-analyzer --version` compares itself with this plugin's
# `shared/min-analyzer-version` and prints `ANALYZER-VERSION-` on
# stderr when it is older. Outside a skill nobody names the plugin root, so
# that warning was silent exactly when an agent ran `check` from a plain shell
# after a plugin update. This runs before the early exit below: the project's
# rules copy does not carry it.
#
# Deliberate constraints. Do not relax one without revisiting the decision that
# authorized this hook:
#   - reads no project file beyond the three existence tests above
#   - prints nothing that depends on project content (existence, never bytes:
#     this script must never compare the two copies or report on staleness)
#   - invokes specdev-analyzer only as `--version`, and only at a project root
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

# One line when the analyzer is missing or older than this plugin needs.
version_note() {
    if ! command -v specdev-analyzer >/dev/null 2>&1; then
        echo "specdev: specdev-analyzer is not on PATH; the specdev skills need it. Tell the user once; install it per ${PLUGIN_ROOT}/shared/analyzer-install.md."
        return
    fi
    line="$(SPECDEV_PLUGIN_ROOT="${PLUGIN_ROOT}" specdev-analyzer --version 2>&1 >/dev/null | grep 'ANALYZER-VERSION-' | head -n 1)"
    if [ -n "${line}" ]; then
        echo "specdev: ${line}"
        echo "Tell the user once: the findings may show defects the plugin's release already fixed until they re-install."
    fi
}

dir="$(pwd)"
while :; do
    if [ -f "${dir}/spec.toml" ] && [ -d "${dir}/specs/specdev" ]; then
        version_note
        # The project owns a copy; Claude Code already loaded it. Stay quiet.
        if [ -f "${dir}/.claude/rules/specdev.md" ]; then
            exit 0
        fi
        cat "${RULES}"
        exit 0
    fi
    # Repository boundary. Checked AFTER the marker test, never before: a
    # specdev project normally has its own `.git` at the same directory as
    # `spec.toml`, and stopping there first would mean never matching anything.
    # This only fires when we are about to climb ABOVE a repository without
    # having found the markers — i.e. the session started inside a vendored or
    # nested project that is not spec-managed. Without it, such a session got
    # the enclosing project's rules, which describe a tree it is not in.
    if [ -e "${dir}/.git" ]; then
        exit 0
    fi
    parent="$(dirname "${dir}")"
    [ "${parent}" = "${dir}" ] && break
    dir="${parent}"
done

exit 0
