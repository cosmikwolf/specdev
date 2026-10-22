# spec-reverse — single-module mode

Entered from the main SKILL.md when `$ARGUMENTS` is a bare module name (not `--all`). Produces exactly one module TOML under `specs/modules/`.

## Single-module reverse

Entered after the main SKILL.md's Steps 0–2 have run (env preamble, `analyzer reverse` pre-flight, authoritative guides read).

1. Read `specs/architecture.toml` when it exists. If it is missing, stop only when the profile's module schema requires a field whose value names an architecture entry (`firmware-layered`: `layer`); then say *"Run `/spec-reverse` (no args) first to produce architecture."* Otherwise continue without it — `validate` reports a missing `architecture.toml` as info (`LAYOUT-005`) only, and a `library-api` member of a workspace normally has none.
2. Read every existing `specs/modules/**/*.toml` for cross-module context.
3. Check if `specs/modules/**/<ModuleName>.toml` exists; if so, ask **Compare / Replace / Cancel**.
4. Locate source for the module. Heuristics:
   - Rust: `src/<snake_case>.rs`, `src/<snake_case>/mod.rs`, or a workspace member with that name.
   - C/C++: `src/<snake_case>.c`, `src/<snake_case>.cpp`, plus matching header.

   If not found at expected paths, ask the user.

5. **Author a stub `specs/modules/<ModuleName>.toml`** with the bare minimum required by the project's profile (typically `name`, plus required metadata fields the schema demands). The stub exists so `extract-items` can resolve a source file for this module.

6. Run `specdev-analyzer extract-items --module <ModuleName>` from the project root. The YAML output enumerates every top-level item in the source file (kind, name, attributes, span). Use it as the structural basis for filling out the rest of the spec — it is authoritative about what's in the source.

### Extraction checklist

For each item, quote the source lines used as evidence. Fields not evidenced are omitted.

- **`name`** — PascalCase derived from the file/module name.
- **`layer`** — infer from dependencies: direct register access → `pac` or `driver`; depends only on drivers → `middleware`; depends on middleware or has policy/business logic → `application`.
- **`description`** — one sentence stating the module's *purpose* at a level higher than the rest of the spec. Do not restate anything already captured by other fields (no listing of actions, state machines, ports, hardware, dependencies, or error variants). If the sentence could be deleted without losing information a reader cannot get from scanning the rest of the file, rewrite it. Target ≤ 20 words. Good example: *"Drives the onboard status LED with solid, blinking, and breathing patterns."* Bad: *"A driver module in the driver layer that exposes set_pattern and tick actions, owns GPIO pin PA5, uses a critical section for shared state, and implements a three-state FSM for the LED."*
- **`depends_on`** — imports/includes to other in-project modules.
- **`actions[]`** — each public function (`pub fn`, non-`static` C function, or exported C++ method). For each: `params` (inline if ≤3), `direction` from parameter mode (`in` / `out` / `inout`), `blocking` (true if it can await, loop on IO, or acquire a mutex — otherwise false), `isr_safe` (true only when the function is clearly safe from ISR context).
- **`state_machines[]`** — any enum consumed by a `match`/`switch` dispatching on current state; states are enum variants. Event enum, initial state, and transitions if the source dispatch is clear.
- **`ports[]`** — each trait the module implements (Rust) or vtable/PIMPL interface (C++) or function-pointer struct (C).
- **`hardware`** — pins, peripherals, clocks the module initializes or references. Cross-check against `specs/architecture.toml`.
- **`errors[]`** — variants of the module's error enum, or distinct return codes with stable names.
- **`metadata.MemoryModel`** — `allocation` ∈ {`static_alloc`, `heap_alloc`, `stack_alloc`}. Static allocations, `static mut`, `Box::leak`, fixed-size arrays → `static_alloc`. `Vec::new()`, `malloc`, `new` → `heap_alloc`.
- **`metadata.ConcurrencyModel`** — `protection` ∈ {`none`, `critical_section`, `mutex`, `atomic`, `disable_interrupts`}. Infer from the code's guards.
- **`metadata.ErrorHandling`** — `strategy` (panic vs return vs defer) and whether errors propagate.
- **`metadata.Ownership`** — `owner` module if any.

### Writing

Present the candidate TOML with LOW-confidence annotations next to LOW fields only (per `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`, inline `# LOW: inferred, please confirm` markers). Wait for confirmation, then write to `specs/modules/<ModuleName>.toml`.

Run `specdev-analyzer validate --no-color`; if errors, show them and ask whether to iterate. For unfamiliar rule names, read the project's `specs/specdev/domain.toml` `[[project.graph_checks]]` blocks for the rule's `kind` and meaning.

Report: source files analyzed, field confidence summary, action/state/error counts, validation result.
