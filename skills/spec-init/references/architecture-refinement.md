# spec-init — illustrative architecture refinement per shipped profile

Step 6 in `/spec-init` reads `specs/architecture.toml` after bootstrap and offers per-profile refinements. The bootstrap-emitted templates are valid as-is; this is **interview-driven refinement, not error recovery** — only edit fields the user actually wants to change.

The shapes below are **illustrative, not exhaustive** — only `firmware-layered` and `library-api` have a section here, because those two have refinement shapes worth spelling out. For **every other profile** — `rust-app`, `audio-plugin`, `rust-workspace`, `flutter-app`, `nextjs-app`, and any user-authored one — read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md` and the emitted `specs/architecture.toml` together — the README enumerates which architecture-template fields the profile considers required vs optional, and the emitted template carries commented hints. Apply the same "only edit what the user actually wants to change" rule.

## firmware-layered

Pre-filled, confirm or change:

- `[platform].target_triple` — `thumbv8m.main-none-eabihf` (Cortex-M33 / STM32H5). Common alternatives: `thumbv7em-none-eabihf` (M4F/M7F; STM32F4/H7), `thumbv6m-none-eabi` (M0/M0+).
- `[platform.mcu_core]` / `[platform.mcu_runtime]` — `cortex-m` / `cortex-m-rt`.
- `[runtime.panic_handler]` — `panic-probe`.
- `[logger.provider]` / `[logger.backend]` — `defmt` / `defmt-rtt`.

Commented in the template; uncomment + set if the user knows the part:

- `[platform].mcu_part` (e.g., `"stm32h563"`, `"stm32f411"`, `"nrf52840"`) — drives the inlined MCU block's HAL feature flag at codegen time. Optional in the schema, but strongly recommended; without it, codegen emits a `// SPEC WARNING` comment.

Optional (offer but don't insist):

- `[runtime.allocator]` (e.g., `embedded-alloc`) — only if modules need `alloc`.
- `[runtime.critical_section_impl]` — only when overriding `cortex-m`'s bundled impl.
- `[[external_deps]]` — project-wide crates beyond the typed slots (e.g., `heapless`, `static-cell`).

## library-api

The library-api architecture template is minimal — no required fields. Offer:

- `msrv = "1.75"` — minimum supported Rust version, becomes `Cargo.toml`'s `rust-version`.
- `[[features]]` entries — Cargo features the library exposes.

## Optional `[project]` fields (any profile)

Write to `spec.toml` only if the user asks:

- `edition` — `"2018"`, `"2021"` (default), or `"2024"`.
- `license` — SPDX identifier (e.g., `"MIT OR Apache-2.0"`).
- `description` — one-line, ≤200 chars (libraries should set this; it appears on crates.io).
