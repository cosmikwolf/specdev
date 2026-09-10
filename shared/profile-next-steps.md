# Suggested next steps after bootstrap, by profile

Used by `/spec-init` to render per-profile guidance after a successful `specdev-analyzer bootstrap`.

For any profile not listed below (including user-authored profiles), read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md` — the README is authoritative for what to do next.

## firmware-layered

Inline at least one MCU block + a runtime block + the matching bridge before authoring modules.

List shipped MCU/runtime blocks with `ls "${CLAUDE_PLUGIN_ROOT}/blocks/"` and shipped bridges with `ls "${CLAUDE_PLUGIN_ROOT}/bridges/"`; pick the pair matching the project's MCU and runtime.

Example for STM32H5 + Embassy:

```
/spec-inline stm32h5-rs && /spec-inline embassy && /spec-inline-bridge embassy+stm32h5-rs
```

Then `/spec-design <one-line module description>`.

## library-api

Go straight to `/spec-design <one-line module description>` to add the first module.

## audio-plugin

Inline an audio framework block (e.g. `nih-plug`) and optionally a UI block (e.g. `egui`) plus the matching bridge if the framework + UI are paired:

```
/spec-inline nih-plug && /spec-inline egui && /spec-inline-bridge egui+nih-plug
```

Then `/spec-design <plugin shell module>`.

## flutter-app

Go straight to `/spec-design <one-line module description>` — start with the composition root (an `app`-role module), then screens and services. If the app consumes a Rust library over FFI or needs BLE, inline the matching block once it ships (list with `ls "${CLAUDE_PLUGIN_ROOT}/blocks/"`).

Platform scaffolding (`flutter create . --platforms <matrix>`) is a one-time manual step before the first `/spec-gen` compile gate; the platform directories it writes are externally owned and never generated. Delete the scaffold's `test/widget_test.dart` afterwards — it references template symbols a spec-generated app doesn't have.

## nextjs-app

Go straight to `/spec-design <one-line module description>`. Start with the composition root — an `app`-role module, exactly one per project, which owns `app/layout.tsx` — then one `page`, then the `service` modules behind it.

Every module carries a required `role`, one of `app`, `layout`, `page`, `route_handler`, `boundary`, `component`, `server_action`, `service`, `model`. A `service` also carries a required `environment` (`server` / `client` / `universal`) — the emitter reads it to choose the `server-only` or `client-only` guard import, so it is never defaulted. A `page`, `layout`, `boundary` or `component` may carry `client_component = true`; a `service` may not.

You never author a file under `app/`. The route files are **derived**: `specdev-analyzer derive --write` re-exports each module body into the `app_path` the module declares, and `/spec-gen` runs it. Author the body under `[paths] source`; the URL is derived from `app_path`, never stored.

Two gates ship in the registry (`specs/specdev/profile.yaml` `gates:`) and `/spec-gen` Step 8 runs both through `specdev-analyzer gate`: `fast` (module scope — `pnpm install`, `next typegen`, `tsc --noEmit`, `vitest run --passWithNoTests`) and `next-build` (batch scope — `next build`). `next build` is the only one of the two that catches a client/server rule violation in a route file; `typegen` and `tsc` are blind to it. Keep at least one `scope: module` gate: with none declared, `/spec-gen` refuses to generate at all.

`tsconfig.json`, `next.config.ts`, `vitest.config.ts`, `playwright.config.ts` and `.gitignore` are written by `specdev-analyzer manifest --write` **only when absent**, then externally owned — edit them freely, but know that each is in the gate key, so an edit flips every module to `verified: false` and the next gen re-gates them. `package.json` is the opposite: pure output, rewritten on every `manifest --write`; add dependencies to `architecture.toml [[external_deps]]` or a module's `external_deps[]`, never to the file.

## rust-workspace

Bootstrap each member crate with its own profile, passing the language the member's profile declares (`language:` in `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/profile.yaml`):

```
cd <member> && /spec-init --non-interactive --profile <profile> --language <profile language> --name <member-name>
```

## Other / user-authored profile

Read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md`; the README is the authoritative source for what to do next.
