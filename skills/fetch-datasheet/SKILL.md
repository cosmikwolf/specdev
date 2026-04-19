---
name: fetch-datasheet
description: Download, parse, and summarize a hardware datasheet PDF into docs/datasheets/<part>/<doc_type>/summary.yaml.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, WebSearch, WebFetch
argument-hint: <part-number | URL | description>
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /fetch-datasheet — Download, parse, summarize hardware datasheets

You are fetching a hardware datasheet, parsing it into chapters, and generating a structured summary that downstream skills can cite. Follow these steps in order. Never invent values — when a field is not stated in the parsed text, write `null`.

---

## Step 1 — read context

Read `specs/architecture.toml` if it exists — note the MCU target so you can suggest sensible defaults. If it doesn't exist, proceed anyway; datasheets are useful even pre-architecture.

## Step 2 — determine input from `$ARGUMENTS`

- **Empty** — ask the user what to fetch. Suggest the MCU name from `specs/architecture.toml` if available.
- **Direct URL** (starts with `http` and ends in `.pdf`) — proceed to Step 4.
- **Part number or description** — proceed to Step 3.

## Step 3 — search for datasheet

Search the web: `"<part> datasheet filetype:pdf"`. Prefer manufacturer-hosted PDFs (espressif.com, st.com, ti.com, nxp.com, microchip.com, infineon.com, nordicsemi.com, analog.com). Present the top candidate URLs and ask the user to confirm before downloading.

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

Write `docs/datasheets/<part_name>/<doc_type>/summary.yaml` with structured hardware data:

- `part`, `manufacturer`, `document_type`, `source_url`, `source_file`.
- `overview` — `description`, `core`, `max_clock_mhz`, `flash`, `sram`.
- `peripherals` — list of `{name, count, notes}`.
- `pins` — `total_gpio`, `adc_pins`, `dac_pins`, `touch_pins`, `strapping_pins`.
- `electrical` — `vdd_range`, `io_voltage`, `max_gpio_current_ma`, `deep_sleep_current_ua`.
- `timing` — `startup_time_ms`, `clock_sources`.
- `register_maps` — `{peripheral, chapter_file, base_address}`.
- `interfaces` — `{name, max_clock_mhz, modes}`.
- `power_modes` — `{name, typical_current_ma}`.
- `notes` — free-form.

**Critical:** only populate fields clearly stated in the parsed text. Use `null` for missing data. Never invent or guess. Downstream skills will trust this file to constrain spec authoring.

## Step 8 — report

Output summary: source URL, PDF path, parser results, chapters parsed, images extracted, summary fields populated/total, peripherals found, register maps referenced. Suggest next steps: run `/spec-design` citing this summary, fetch additional related documents (errata, programming guide), or review `summary.yaml` for gaps.
