# spec-init — domain `init.md` DSL template

The `init.md` file at `domains/<domain-name>/init.md` is loaded by `/spec-init` Step 4b after the user selects that domain. It declares the domain-specific portion of project scaffolding using five required H2 sections. Use `"None."` as the section body when a section contributes nothing.

This file is a **template** for authors adding a new domain. The two production domains (`firmware` and `egui`) are labeled and linked below for contrast; both use minimal bodies.

## Skeleton

Copy the block below into a new `domains/<domain-name>/init.md`, then replace placeholders.

~~~markdown
# <domain-name> — spec-init extensions

This file is loaded by `/spec-init` after the user selects the `<domain-name>` domain. It declares the domain-specific portion of project scaffolding.

## Extra interview questions

None.

## architecture.toml skeleton

Emit an empty file.

## Extra scaffold directories

None.

## Extra scaffold files

None.

## Post-write notes

None.
~~~

## Real examples for contrast

- **Firmware:** `domains/firmware/init.md` — all sections minimal; firmware populates architecture and per-module hardware later via `/spec-design`.
- **Egui:** `domains/egui/init.md` — identical shape, except `Extra scaffold directories` declares `specs/stores/` with a `.gitkeep` policy. Illustrates how a domain adds a top-level spec directory.

Open both to see how the same DSL supports very different domains.

## Section semantics

### Extra interview questions

Numbered list of questions to ask after the generic `spec.toml` interview, with each question's schema destination. `"None."` when every architecture-schema slot is populated later by `/spec-design`.

Hypothetical populated example for a synthetic `robotics` domain:

~~~markdown
## Extra interview questions

1. **Drivetrain kinematics model** — one of `differential`, `ackermann`, `holonomic`. Destination: `[kinematics] model` in `architecture.toml`.
2. **Safety envelope policy** — one of `hard-stop`, `graceful-stop`, `reduce-speed`. Destination: `[safety] envelope_policy` in `architecture.toml`.
~~~

### architecture.toml skeleton

Either literal TOML to write to `specs/architecture.toml`, or the sentinel `"Emit an empty file."`. Include `#:schema` directives only if the domain's architecture schema requires them.

### Extra scaffold directories

Bulleted list of directories to create under the project root, each with a gitkeep / starter-content policy. Example from egui:

~~~markdown
- `specs/stores/` — one TOML per shared state container. Write a `.gitkeep` so the empty directory survives git.
~~~

### Extra scaffold files

Bulleted list of files to create with literal content (typically domain-specific README stubs or module templates). `"None."` is common.

### Post-write notes

Free-form prose shown to the user after scaffold completion. Surface manual steps, follow-up conventions, or domain quirks. `"None."` is acceptable.

## Adding a new domain — checklist

1. Create `domains/<domain-name>/domain.toml` (schemas, layer order, `[validation.rules]`).
2. Create `domains/<domain-name>/init.md` from the skeleton above.
3. Create `domains/<domain-name>/author.md` (structural conventions consumed by `/spec-design`).
4. If the domain supports validation with domain-specific rule ids, create `domains/<domain-name>/validate.md` with rule-id explanations.
