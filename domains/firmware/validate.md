# Firmware domain — validation reasoning

Checks beyond what the JSON Schema and graph-check primitives can express.
The `/spec-validate` skill runs through these after
`specdev-analyzer validate` finishes, producing extra findings the analyzer
cannot catch without semantic understanding.

Each check lists:
- **Trigger** — when it runs
- **Finding** — what to report
- **Severity** — `error`, `warning`, or `info`

If a check cannot run (missing data, external file unavailable), note it and
move on; do not block other checks.

---

## 1. ISR-safe vs blocking (ISR060)

**Trigger:** every `action` on every module.

**Check:** if `isr_safe = true` and `blocking = true`, fail.

**Finding:** "Action `<Module>.<action>` claims ISR safety but is marked
blocking. ISR-safe functions must return within bounded time without
waiting on hardware, DMA, or locks."

**Severity:** error.

Note: the schema alone cannot catch this — both fields are independent
booleans. The graph-check primitive set does not currently express
conditional constraints like "if A=true then B=false". Promote to the
analyzer when the pattern repeats.

---

## 2. ISR-safe transitive blocking

**Trigger:** any action with `isr_safe = true`.

**Check:** for every module in the action's implied call graph (use
`depends_on` as an approximation), if any action on a dependency is
`blocking = true`, the original action can transitively block.

**Finding:** "Action `<Module>.<action>` claims ISR safety but transitively
calls blocking code in `<Dep>.<action>`. Either the dependency is unused in
this path (tighten `depends_on`) or the ISR-safety claim is wrong."

**Severity:** warning — the dependency graph is coarser than the real call
graph.

---

## 3. Error-handling vs concurrency consistency

**Trigger:** every module.

**Checks:**

- `concurrency.isr_safe = true` AND `error_handling.strategy = "panic"` →
  warning. ISR panic on embedded targets is effectively a system reset.
- `concurrency.thread_safe` implied true (`protection != "none"`) AND
  `error_handling.strategy = "errno"` → warning. Without thread-local
  storage, errno races across threads.

**Severity:** warning.

---

## 4. Shared-state vs protection consistency

**Trigger:** module has `ConcurrencyModel.shared_state` non-empty.

**Checks:**

- `protection = "none"` → warning. Declared shared state without protection
  is a data race.
- `protection = "atomic"` but any entry in `shared_state` names a complex
  value (buffer, multi-field struct) → warning. Atomics fit single scalars.
- Module has `protection` set (non-`none`) but `shared_state = []` →
  info. The protection may be unused; consider dropping.

**Severity:** warning unless otherwise noted.

---

## 5. Constraint vs interface contradiction

**Trigger:** module declares a `Timing` metadata category.

**Checks:**

- `deadline_us` is set AND an action is `blocking = true` with no timeout
  parameter in its `params[]` → warning. The action cannot honor the
  deadline; either add a timeout parameter or loosen the deadline.
- `MemoryModel.allocation = "static_alloc"` AND an action's `returns`
  implies heap allocation (`Box<T>`, `Vec<T>`, `String`, or non-fixed types)
  → warning.

**Severity:** warning.

---

## 6. External dependency verification

**Trigger:** module declares `external_deps[]`.

**Check:** each `external_deps[].name` should appear in the project's
`spec.toml` `[project.external_packages]` or in a peer module's
`external_deps` for shared libraries. Libraries used only inside one module
are acceptable, but repeated use of an undeclared library flags drift.

**Severity:** info.

---

## 7. Transitive dependency completeness

**Trigger:** action parameter or port uses a type defined in a module that
is NOT in this module's `depends_on[]`.

**Finding:** "Action `<Module>.<action>` accepts a `<Type>` defined in
`<OwnerModule>`, which is not in this module's `depends_on[]`. Either add
the dependency or remove the direct use of `<Type>`."

**Severity:** warning.

---

## 8. Ownership conflicts (OWN050)

**Trigger:** project validate, every hardware.pins[] / peripherals[] /
clocks[] entry.

**Checks:**

- Same `peripheral.name` in two modules' `hardware.peripherals[]` → error.
- Same `(port, pin)` pair in two modules' `hardware.pins[]` → error.
- Same `clock.name` in two modules' `hardware.clocks[]` → error.

**Finding:** "Hardware resource `<resource>` is declared in both
`<ModuleA>` and `<ModuleB>`. One module must own it; the other should
receive access through an action parameter."

**Severity:** error.

---

## 9. Datasheet constraints

**Trigger:** project has `docs/datasheets/*.md` summaries available.

**Check:** For each peripheral a module owns, cross-reference its
datasheet-derived constraints (max SPI clock, voltage range, timing):

- Module's relevant `metadata` or action parameters contradict a datasheet
  value → warning with the specific fact and source.

**Severity:** warning. Datasheet summaries are advisory; authors may have
additional context.

---

## 10. Requirement coverage

**Trigger:** project validate, every requirement id in
`specs/requirements/**`.

**Checks:**

- Requirement id appears in zero module's `satisfies[]` → REQ001 warning.
- Requirement's `verified_by[]` lists `Module.action` pairs that do not
  resolve to existing actions → warning.
- Requirement's `source[]` lists ids that do not resolve to any
  `[[source]]` entry across the requirements files → REQ003 error.
- Requirement has `source_ref` set but `source = []` (or unset) → warning.
  A free-form locator with no source to anchor it is ambiguous.
- A declared `[[source]]` that no requirement references → info. May be
  a stub waiting on derivation, or dead metadata.

**Severity:** warning unless otherwise noted.

---

## 11. Layer dependency sanity (sanity check for LAYER001)

**Trigger:** after analyzer reports LAYER001.

**Finding format:**
```
LAYER VIOLATION: <Module> (layer: <layer>) depends on <Dep> (layer: <dep_layer>)
  Allowed layers for <layer>: [<allowed_deps list>]
  Fix: raise <Dep> to a higher layer, or introduce a thin wrapper in an allowed layer.
```

**Severity:** error (this is a pre-formatted restatement of analyzer output
to help the human architect act on it).

---

## 12. Background-thread stack budget

**Trigger:** any module declares a `@BackgroundThread`-like pattern
(future metadata category; placeholder until added).

**Check:** sum declared stack sizes; if `spec.toml [target].ram_kb` is set
and the total exceeds 50% of RAM, warn with a per-thread breakdown.

**Severity:** warning.

**Note:** not wired up until the `BackgroundThread` metadata category
lands. Keep the placeholder so Phase 6+ knows the rule is planned.

---

## 13. Drift detection (audit pipeline)

See `/spec-validate` for spec↔source drift. The audit pipeline compares
extracted YAML against the language adapter's view of the source tree (see
`specdev-analyzer audit`). Drift categories:

- Interface drift (extra / missing / renamed actions).
- Error-variant drift.
- Concurrency drift (shared state, protection mismatches).
- Memory drift (allocation method, instance counts).

If any drift is found: the remediation is to update the spec, update the
code, or explicitly accept the drift (future: `# drift: accepted` marker).
Generated code is never edited directly — the spec is authoritative.

---

## How to report findings

For each finding:

```
<rule id or "reasoning">  <severity>  <module>[:<action|port|state>]
  <one-sentence summary>
  <one-sentence fix hint>
```

Group by severity, then by module. End with a one-line summary
(`N errors, M warnings, K info`). If any error is present, report the
project as FAILED.
