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

Platform scaffolding (`flutter create . --platforms <matrix>`) is a one-time manual step before the first real build; the platform directories it writes belong to Flutter tooling. Delete the scaffold's `test/widget_test.dart` afterwards — it references template symbols the app doesn't have.

## nextjs-app

Go straight to `/spec-design <one-line module description>`. Start with the composition root — an `app`-role module, exactly one per project, which owns `app/layout.tsx` — then one `page`, then the `service` modules behind it.

Every module carries a required `role`, one of `app`, `layout`, `page`, `route_handler`, `boundary`, `component`, `server_action`, `service`, `model`. A `service` also carries a required `environment` (`server` / `client` / `universal`) — it decides the `server-only` or `client-only` guard import the body starts with, so it is never defaulted. A `page`, `layout`, `boundary` or `component` may carry `client_component = true`; a `service` may not.

Route files under `app/` are two-line shims that re-export the module body at the module's `app_path` (the profile README shows the three forms). Write the body under `[paths] source`; the URL follows from `app_path`, never stored.

Two gates ship in the registry (`specs/specdev/profile.yaml` `gates:`), and `scripts/specdev-gates.sh` runs them after `specdev-analyzer check`: `fast` (`pnpm install`, `next typegen`, `tsc --noEmit`, `vitest run --passWithNoTests`) and `next-build` (`next build`). `next build` is the only one of the two that catches a client/server rule violation in a route file; `typegen` and `tsc` are blind to it.

`tsconfig.json`, `next.config.ts`, `vitest.config.ts`, `playwright.config.ts`, `.gitignore`, `pnpm-workspace.yaml` and `.github/workflows/specdev.yml` were written by `bootstrap` **only when absent**, and are the project's now. `package.json` is the project's too: add dependencies there.

## rust-workspace

Bootstrap each member crate with its own profile, passing the language the member's profile declares (`language:` in `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/profile.yaml`):

```
cd <member> && /spec-init --non-interactive --profile <profile> --language <profile language> --name <member-name>
```

## Every profile

Before writing a module's code, run `specdev-analyzer context --module <Module>` and `specdev-analyzer report decisions --module <Module>` and read what they list; mark a module without code `planned = true`. CI needs the repository variable `SPECDEV_ANALYZER_REPO` and the secret `SPECDEV_ANALYZER_TOKEN` that `.github/workflows/specdev.yml` reads, and a committed lockfile.

## Other / user-authored profile

Read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md`; the README is the authoritative source for what to do next.
