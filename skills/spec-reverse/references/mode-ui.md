# spec-reverse — UI mode

Entered from the main SKILL.md when `$ARGUMENTS` is `--ui`. Produces `specs/ui/*.toml` from UI-related source code.

## Step 5 — UI reverse

1. Read `specs/architecture.toml` and every `specs/modules/**/*.toml`.
2. If `specs/ui/*.toml` exists, ask **Compare / Replace / Merge**.

Scan for UI code:

- **Display** — driver chip, resolution, interface (SPI/I2C/parallel), color depth from init code.
- **Input devices** — GPIO pin + event kind (press / long-press / rotate) + debounce.
- **Indicators** — LEDs, per-state color, pattern.
- **Screens** — each rendering function is a candidate screen; elements are `text` / `image` / `shape` calls with data bindings.
- **Navigation** — transitions between screens, timeouts, global actions (e.g. long-press to sleep).

Read `<Schema dir>/ui.schema.yaml` for the exact field list. Present candidate `specs/ui/*.toml` with LOW-confidence inline markers (per `specdev/shared/comment-policy.md`). Write on confirmation.

Any `description` field (on the UI file, a screen, or an element) is one sentence describing *purpose*. Do not summarize widgets, inputs, or navigation defined elsewhere in the same file.

Run validate. Report file count, hardware / input / screen / nav counts, validation result. For unfamiliar rule ids see `specdev/shared/rule-ids.md`.
