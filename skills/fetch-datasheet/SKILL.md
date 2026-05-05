---
name: fetch-datasheet
description: This skill should be used when the user asks to "fetch a datasheet", "download the datasheet for <part>", "parse this chip PDF", or "summarize this reference manual". Downloads, parses, and summarizes a hardware datasheet PDF into docs/datasheets/<part>/<doc_type>/summary.yaml.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, WebSearch, WebFetch
argument-hint: <part-number | URL | description>
---

# /fetch-datasheet — Download, parse, summarize hardware datasheets

Fetch a hardware datasheet, parse it into chapters, and generate a structured summary that downstream skills can cite. Follow these steps in order. Never invent values — when a field is not stated in the parsed text, write `null`.

---

## Step 1 — read context

Read `specs/architecture.toml` if it exists — note the MCU target to suggest sensible defaults. If it does not exist, proceed anyway; datasheets are useful even pre-architecture.

## Step 2 — determine input from `$ARGUMENTS`

- **Empty** — ask the user what to fetch. Suggest the MCU name from `specs/architecture.toml` if available.
- **Direct URL** (starts with `http://` or `https://`) — proceed to Step 4. Query strings and content-disposition URLs without a `.pdf` extension are accepted; Step 5's `%PDF` magic-bytes check rejects anything that isn't actually a PDF.
- **Part number or description** — proceed to Step 3.

## Step 3 — search for datasheet

Use `WebSearch` with the query `"<part> datasheet filetype:pdf"` to find candidates. Prefer manufacturer-hosted PDFs (espressif.com, st.com, ti.com, nxp.com, microchip.com, infineon.com, nordicsemi.com, analog.com). Present the top candidate URLs and ask the user to confirm one before proceeding — do not use `WebFetch` at this stage. `WebFetch` is reserved for Step 5's download of the confirmed PDF URL.

## Step 4 — determine metadata

From URL, filename, or user input, determine:

- **part_name** — lowercase, hyphenated (e.g. `esp32-s3`, `stm32f411`, `nrf52840`).
- **doc_type** — one of: `datasheet`, `technical_reference`, `errata`, `programming_guide`, `user_guide`.

If ambiguous, ask. Default to `datasheet`.

## Step 5 — create directory and download

Create `docs/datasheets/<part_name>/<doc_type>/` if it does not exist. If it already exists with an existing PDF, warn and ask whether to overwrite or pick a different name.

Download the PDF to `docs/datasheets/<part_name>/<doc_type>/<filename>.pdf`.

Verify: file size > 0, and the first four bytes are `%PDF`. If verification fails, stop and report the error — do not attempt to parse garbage.

## Step 6 — run parser

This step requires `uv` (Astral's Python runner). If `uv` is not on PATH, tell the user to install it (`curl -LsSf https://astral.sh/uv/install.sh | sh`, or `brew install uv`) and stop. As a fallback for users who do not want `uv`, `python3 ${CLAUDE_PLUGIN_ROOT}/scripts/parse_datasheet.py` works if the script's dependencies are already installed in the active Python environment — but `uv run` is the supported invocation because it auto-resolves the script's inline dependency declarations.

Execute:

```
uv run ${CLAUDE_PLUGIN_ROOT}/scripts/parse_datasheet.py <pdf_path> --output-dir docs/datasheets/<part_name>/<doc_type>/parsed/
```

Exit codes:

- `0` — continue.
- `1` — partial success (some chapters failed). Continue, surface warnings to the user.
- `2` — fatal error. Stop and report.

## Step 7 — generate summary.yaml

Read parsed chapters from `docs/datasheets/<part_name>/<doc_type>/parsed/` and its `index.yaml`.

Write `docs/datasheets/<part_name>/<doc_type>/summary.yaml`. The full schema, required fields, and authoring rules live in `references/summary-schema.md` — follow it verbatim. Only populate fields clearly stated in the parsed text; write `null` for anything not stated. Never invent values — downstream skills trust this file.

## Step 8 — report

Output summary: source URL, PDF path, parser results, chapters parsed, images extracted, summary fields populated/total, peripherals found, register maps referenced. Suggest next steps: run `/spec-design` citing this summary, fetch additional related documents (errata, programming guide), or review `summary.yaml` for gaps.

## Additional resources

- `references/summary-schema.md` — full schema for the emitted `summary.yaml`, including required fields and null-handling rules.
