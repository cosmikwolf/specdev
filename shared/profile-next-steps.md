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

## rust-workspace

Bootstrap each member crate with its own profile, passing the language the member's profile declares (`language:` in `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/profile.yaml`):

```
cd <member> && /spec-init --non-interactive --profile <profile> --language <profile language> --name <member-name>
```

## Other / user-authored profile

Read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md`; the README is the authoritative source for what to do next.
