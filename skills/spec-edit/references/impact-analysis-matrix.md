# spec-edit — impact analysis matrix

Step 5 of `/spec-edit` audits downstream effects before writing. The cone's `related_modules` block (`upstream` / `downstream` / `port_endpoints` / `binding_endpoints` / `hardware_owners` / `re_export_targets` / `contains` / `contained_by`) is the bounded input set — these are the modules that a change to the target may break.

For each change type the user proposes, surface findings grouped under the matching row.

## Change type → what to check

- **Renaming/removing an action** — grep `depends_on` and source under `[paths].source` to find call sites. List broken call sites.

- **Renaming/removing a port** — grep `specs/architecture.toml` `[[connections]]` for `from = "<Module>.<port>"` or `to = "<Module>.<port>"`. List broken wires.

- **Changing `layer`** (firmware-layered) — check every module that `depends_on` this one; if any caller's layer is now strictly above the target's per the layered ordering (`application > middleware > driver > HAL > PAC`), flag a future `layer_dependency` violation. Same in reverse: if this module now depends on a higher layer, flag.

- **Changing `depends_on`** — check for cycles by building the new graph mentally.

- **Changing `hardware.peripherals` / `hardware.pins`** (firmware-layered) — grep every module for the same resource. Two modules owning the same pin/peripheral surfaces as a hardware-ownership violation. Cross-check `architecture.toml [[connections]]` for endpoints that name the renamed/removed peripheral.

- **Changing `metadata.ConcurrencyModel`** — local check: if any action is `isr_safe = true`, `protection` cannot be `none`.

- **Changing `bindings.<a>+<b>`** — confirm referenced `task` / `peripheral` names still exist on the target module after the edit; cross-reference any other bindings that may reference removed entries.

- **Changing `visibility`** (library-api) — modules whose `[[items]]` `kind = "re_export"` references this module may need their re-export `visibility` adjusted.


## Reporting

If any issue surfaces, present them grouped by change-type row above and **wait for user confirmation** before editing. Do not silently accept a change that breaks downstream callers — the user decides whether to fix the dependents or revise the proposed edit.

- **Editing a module whose `contains` list is not empty** (a composition root, e.g. the nextjs-app `app` module that owns the root layout) — every module in `contains` renders inside it, though none names it in `depends_on`. A change to what it provides (providers, fonts, metadata defaults, the frame around `children`) reaches all of them; list them. Changing its `role` away from the containing role removes those edges, and the profile's count check (e.g. `single_app_module`) reports the missing root.
