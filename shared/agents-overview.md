# agents/

Subagents shipped by the specdev plugin. Files directly under this directory are auto-discovered by Claude Code.

## `.deferred/`

Subdirectories whose names start with `.` are skipped by the loader. `.deferred/` is used to stage agent designs that aren't yet wired into a live skill workflow — the files are kept under version control for review and iteration but are not user-invocable. Promote an agent by moving its `.md` file out of `.deferred/` into `agents/` once the calling skill is ready.
