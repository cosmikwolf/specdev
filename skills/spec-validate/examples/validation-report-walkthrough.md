# spec-validate — annotated report walkthrough

**Profile in this walkthrough:** `firmware-layered`. Rule names and severities come from the project's `specs/specdev/domain.toml` `[validation.rules]` table — read that file for the authoritative list. The *presentation pattern* shown below is profile-agnostic.

## Raw analyzer output (illustrative findings, real line shape)

The findings below are invented for the walkthrough; the **line shape is not**. Each finding is one
line — lowercase severity, colon, bracketed rule, absolute path (no `#` pointer), colon, message —
and the tail is a count line whose error count alone drives the exit code.

```
$ specdev-analyzer check --no-color
error: [layer_dependency] /path/to/proj/specs/modules/AudioPipeline.toml: module `AudioPipeline` (layer=middleware) depends on `FlightController` (layer=application) — middleware cannot depend on application.
error: [SCHEMA] /path/to/proj/specs/modules/I2sOutput.toml: required property `metadata.ConcurrencyModel` missing.
error: [connection_endpoints_resolve] /path/to/proj/specs/architecture.toml: connection endpoint `AudioPipeline.led_command` does not name a known port.
error: [requirements_have_satisfiers] /path/to/proj/specs/requirements/REQ-AUDIO-003.toml: requirement `REQ-AUDIO-003` not in any module's `satisfies[]`.

loaded 9 modules / 12 requirements / 9 sources / 12 graph checks
4 error(s), 0 warning(s), 14 info(s)
```

Exit 1, because the error count is non-zero.

## Step 3 presentation pattern

Group by category, then severity. Do not dump raw output — translate each rule into a one-line explanation and a concrete fix hint. The rule's `kind` (`layer_dependency`, `endpoints_resolve`, etc.) lives in `specs/specdev/domain.toml`'s `[[project.graph_checks]]` blocks; the rule's severity lives in `[validation.rules]`. Read both for any unfamiliar rule.

### Schema errors (1)

**Required field missing (SCHEMA)** — `I2sOutput` is missing `metadata.ConcurrencyModel`. firmware-layered's `module.schema.yaml` requires the four metadata sub-objects. Use `/spec-edit I2sOutput` to add it.

### Graph-check errors (3)

**Layered architecture (`layer_dependency`)** — `AudioPipeline` (middleware) → `FlightController` (application). The layered ordering is `application > middleware > driver > HAL > PAC` (top-down); a middleware module cannot depend on an application module. Fix options: invert the dependency, or move shared types into a lower layer. Use `/spec-edit AudioPipeline` so impact is analyzed.

**Architecture wiring (`connection_endpoints_resolve`)** — `architecture.toml#/connections/2/to` names port `AudioPipeline.led_command`, but no such port exists on that module (the real port is `led_cmd`). Either rename the port in `AudioPipeline.toml` (`/spec-edit AudioPipeline`) or fix the endpoint string directly in `architecture.toml`.

**Unsatisfied requirement (`requirements_have_satisfiers`)** — `REQ-AUDIO-003` declared in `specs/requirements/REQ-AUDIO-003.toml` is not referenced by any module's `satisfies[]`. Either satisfy it in the relevant module or remove the requirement.

## Step 4 handoff (per the skill's read-only contract)

Do not edit files from `/spec-validate`. Tell the user:

- `layer_dependency` on `AudioPipeline` → `/spec-edit AudioPipeline`.
- `SCHEMA` on `I2sOutput` → `/spec-edit I2sOutput`.
- `connection_endpoints_resolve` in `architecture.toml` → edit `specs/architecture.toml` directly (no skill currently owns that file).
- `requirements_have_satisfiers` → `/spec-edit` on whichever module should satisfy the requirement, mark the requirement `planned = true` if nothing is built for it yet, or delete its file.

Re-invoke `/spec-validate` after fixes to confirm a clean report.
