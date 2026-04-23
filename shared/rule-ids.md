# Validation rule id decoder

Shared reference used by `/spec-validate`, `/spec-design`, and `/spec-edit` when translating analyzer output. Rule ids and severities are declared per-domain in `<Domain dir>/domain.toml` under `[validation.rules]` — that file is authoritative. This document summarizes the firmware-domain rule ids and the fix pattern for each, because those are the ones most commonly surfaced.

When a rule id appears that is not listed here, read the loaded domain's `domain.toml` `[validation.rules]` table to recover its severity and meaning. Do not invent rule ids.

## Firmware-domain rule ids

| Rule | Severity | Meaning | Typical fix |
|---|---|---|---|
| `LAYER001` | error | Module `depends_on` a higher layer than its `[layers.allowed_deps]` permits. | Invert the dependency, or move shared types into a lower layer. |
| `LAYER002` | error | Cyclic module dependency. | Break the cycle by extracting shared types or injecting via port. |
| `META010` | error | Required metadata category missing (`MemoryModel`, `ConcurrencyModel`, `ErrorHandling`). | Add the missing `[metadata.<Category>]` table with required keys. |
| `META011` | warning | Metadata value disagrees with the domain default. | Confirm the override is intentional; otherwise use the default. |
| `FSM020` | error | State machine's `initial` is not in `states[]`. | Rename the initial state or add it to `states[]`. |
| `FSM021` | error | Transition references an unknown `from` or `to` state. | Rename the target or add the missing state. |
| `FSM022` | warning | State unreachable from `initial`. | Add a transition that reaches it, or delete the state. |
| `PORT030` | error | Architecture connection endpoint does not exist on the named module. | Fix the port name on the module, or fix the endpoint in `architecture.toml`. |
| `TYPE040` | warning | Action / port type not in the language's `type_map`. | Add the type to the language config or rename to a mapped type. |
| `REQ001` | warning | Requirement not satisfied by any module. | Add the requirement id to a module's `satisfies[]`, or remove the requirement. |
| `REQ002` | error | Module `satisfies[]` references an unknown requirement id. | Fix the id (typo?) or declare the requirement. |
| `REQ003` | error | Requirement `source[]` references an unknown source id. | Fix the id or declare the `[[source]]` entry. |
| `OWN050` | error | Two modules own the same hardware resource (pin, peripheral). | Resolve ownership — exactly one module per resource. |
| `ISR060` | error | Action is `isr_safe=true` and `blocking=true` simultaneously. | Pick one; the combination is incoherent. |
| `PKG040` | warning | Module under `specs/modules/<X>/` where `<X>` is not a declared package. | Declare the package in `spec.toml` or move the file. |
| `PKG041` | info | Module directly under `specs/modules/` in a multi-package project. | Move the file under a `<pkg>/` directory. |
| `PKG042` | warning | `[[packages]].source_path` does not exist on disk. | Fix the path or remove the package entry. |
| `PKG043` | error | `depends_on` / architecture endpoint references an unknown package or module. | Fix the qualified reference. |
| `PKG044` | error | Unqualified reference is ambiguous across multiple packages. | Qualify as `<pkg>::<Module>`. |
| `MOD_DEPTH` | error | Module file nested deeper than one level under `specs/modules/`. | Flatten to `specs/modules/<pkg>/<Module>.toml` or `specs/modules/<Module>.toml`. |

## Usage from skills

- `/spec-validate` presents findings grouped by rule category. Use this table's fix column as the starting point for the Step 3 "suggest fixes" phase.
- `/spec-design` and `/spec-edit` use this table when iterating after validate. If a rule id is unfamiliar, read `<Domain dir>/domain.toml` rather than guessing.
- When the project is not firmware, the rule ids come from the loaded domain's `domain.toml`. The presentation pattern is identical; only the id set differs.
