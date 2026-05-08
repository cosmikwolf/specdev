# audio-plugin: Rust module conventions

These conventions describe how an `audio-plugin` profile module spec
maps to Rust source. They are language-and-architecture-only —
framework specifics (parameter declaration, audio-port wiring, editor
topology) come from the relevant block's `codegen/rust/README.md`,
layered on top.

This profile targets the [nih-plug](https://github.com/robbert-vdh/nih-plug)
audio plugin framework, which builds VST3 / CLAP / standalone artifacts
from a single Rust crate.

## Crate shape

A plugin crate is a `cdylib` (the C-ABI surface VST3 and CLAP both
require) with an optional `[[bin]]` target for the standalone
distribution format. Cargo.toml is **emitted by the analyzer** from the
spec — do not hand-edit it; edit the spec and regenerate.

| `[plugin].formats` | Cargo.toml shape                                                |
|--------------------|-----------------------------------------------------------------|
| no `standalone`    | `[lib] crate-type = ["cdylib"]`                                 |
| with `standalone`  | `[lib] crate-type = ["cdylib"]` + `[[bin]] name = "<plugin>-standalone"` |

The bin path is always `src/bin/standalone.rs`; codegen emits this file
when standalone is in the format set. The bin links the same crate's
`lib`, so the plugin shell's `impl Plugin for X` is reused unchanged.

## File layout

```
src/
├── lib.rs                        # crate root: declares modules, runs nih_export_*!
├── <plugin_shell_module>.rs      # impl Plugin for X, parameter struct, editor wiring
├── <engine_module>.rs            # DSP coordination (or in a sibling `engine` crate)
├── <dsp_node_module>.rs          # leaf DSP units
└── bin/
    └── standalone.rs             # nih_export_standalone!(); only when format set includes standalone
```

For multi-crate workspaces (the recommended layout): the engine lives
in a sibling member crate using the `library-api` profile (`no_std`-
eligible, no plugin concerns); the
plugin crate uses this profile and depends on the engine via Cargo
path/workspace dep. Cross-crate boundary is the engine ↔ plugin_shell
edge — see profile README §"When to use this profile."

The crate root (`src/lib.rs`) declares each top-level module and runs
the format-driven export macros:

```rust
mod my_plugin;
mod synthesis;          // engine module (single-crate layout)
mod oscillator;         // dsp_node module

use my_plugin::MyPlugin;

nih_export_vst3!(MyPlugin);
nih_export_clap!(MyPlugin);
```

Emit one `nih_export_<fmt>!(<PluginShellType>)` line per format in
`[plugin].formats`, in declaration order. `standalone` does NOT use
`nih_export_standalone!` here — that lives in `src/bin/standalone.rs`
(which calls `nih_export_standalone!(my_plugin::MyPlugin)`).

## Layer emission rules

`module.layer` drives where the module lives in the dependency
hierarchy. The four-tier ordering:

| Layer            | Typical content                                          | Allowed deps           |
|------------------|----------------------------------------------------------|------------------------|
| `host_interface` | Reserved — nih-plug-provided types. No user modules.     | —                      |
| `plugin_shell`   | `impl Plugin for X`, parameter struct, editor wiring     | `engine`, `dsp_node`   |
| `engine`         | Voice management, preset state, DSP coordination         | `dsp_node`             |
| `dsp_node`       | Leaf DSP units (oscillators, filters, envelopes)         | (leaf — no deps)       |

The `layer_dependency` graph check rejects upward dependencies. v1
restricts each plugin crate to **one** `plugin_shell` module — multi-shell
per crate is a bounded follow-up. Modules at `host_interface`
are not authored; the value exists in the enum so the layer ordering
has a complete top edge.

Engine modules MAY be `no_std`-compatible — when the engine lives in a
sibling `library-api` crate, that crate carries its own no_std
discipline; the plugin crate (this profile) has no no_std mode (cdylib
links libstd).

## Plugin shell emission

The `plugin_shell` module is the load-bearing one. Its emission
template — populated from `[plugin]` in architecture.toml plus the
`[nih_plug]` block contribution on the same module:

```rust
use nih_plug::prelude::*;
use std::sync::Arc;

pub struct MyPlugin {
    params: Arc<MyPluginParams>,
    // engine + DSP state lives here
}

#[derive(Params)]
pub struct MyPluginParams {
    // parameters from [[nih_plug.parameters]] — block prose drives this
}

impl Plugin for MyPlugin {
    const NAME: &'static str    = "<from [plugin].name>";
    const VENDOR: &'static str  = "<from [plugin].vendor>";
    const URL: &'static str     = "<from [plugin].url, or empty string when omitted>";
    const EMAIL: &'static str   = "<from [plugin].email, or empty string when omitted>";
    const VERSION: &'static str = "<from [plugin].version>";

    const VST3_CATEGORIES: &'static str = /* per-category lookup, see below */;
    const CLAP_FEATURES: &'static [ClapFeature] = /* per-category slice */;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> { self.params.clone() }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Real-time discipline applies here; see "Realtime path" below.
        ProcessStatus::Normal
    }
}
```

`URL` and `EMAIL` are `&'static str` constants; nih-plug requires them
to be set. When `[plugin].url` / `email` is omitted in the spec, emit
the literal `""` empty string (the host displays nothing).

## Per-format constants

`[plugin].category` is a closed enum mapped to per-format constants by
this lookup table. Codegen reads category once and emits all relevant
constants:

| `category`   | `VST3_CATEGORIES`             | `CLAP_FEATURES` slice                                                |
|--------------|-------------------------------|----------------------------------------------------------------------|
| `instrument` | `"Instrument\|Synth"`          | `&[ClapFeature::Instrument, ClapFeature::Synthesizer]`               |
| `fx`         | `"Fx"`                        | `&[ClapFeature::AudioEffect]`                                        |
| `midi-fx`    | `"Fx\|MIDI"`                   | `&[ClapFeature::NoteEffect]`                                         |
| `analyzer`   | `"Fx\|Analyzer"`               | `&[ClapFeature::Analyzer, ClapFeature::AudioEffect]`                 |

Standalone has no category constant — `nih_export_standalone!` consumes
the same `Plugin` impl and shows the plugin under its `NAME` only.

If `formats` does not include `vst3`, omit the `VST3_CATEGORIES` const.
If `formats` does not include `clap`, omit `CLAP_FEATURES`. nih-plug
tolerates absent-but-unused constants; emitting unused ones is fine but
clutters.

## Audio bus declaration

`[audio]` drives the plugin's `AUDIO_IO_LAYOUTS` constant:

```rust
const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
    AudioIOLayout {
        main_input_channels: NonZeroU32::new(<channels for default_channel_layout>),
        main_output_channels: NonZeroU32::new(<channels for default_channel_layout>),
        aux_input_ports: &[/* one NonZeroU32 per [[audio.aux_inputs]] */],
        aux_output_ports: &[/* one per [[audio.aux_outputs]] */],
        names: PortNames {
            layout: Some("<default_channel_layout, capitalized>"),
            ..PortNames::const_default()
        },
    },
];
```

Channel count from `default_channel_layout`: mono=1, stereo=2, quad=4,
five-one=6, seven-one=8.

If `[audio].latency_samples > 0`, override the default `latency_samples`
method:

```rust
fn initialize(&mut self, _audio_layout: &AudioIOLayout, buffer_config: &BufferConfig, _ctx: &mut impl InitContext<Self>) -> bool {
    self.latency_samples = <[audio].latency_samples>;
    true
}
```

## Standalone bin emission

When `formats` contains `standalone`, emit `src/bin/standalone.rs`:

```rust
use nih_plug::prelude::*;
use <crate_name>::MyPlugin;

fn main() {
    nih_export_standalone::<MyPlugin>();
}
```

The bin uses the lib's published plugin type (declared `pub` in
`lib.rs`). Standalone hosts the plugin in a small JACK / CoreAudio
window — useful for local development and CI smoke tests.

## Realtime path discipline

The `process()` callback runs on the host's audio thread. Three rules
are non-negotiable in this profile (and the basis for the deferred
`realtime_path_no_alloc` graph-check):

1. **No allocation.** No `Vec::new()`, `Box::new()`, `String::new()`,
   `format!`, `to_owned`, or any other heap call. Allocate during
   `initialize()` (which runs on the main thread, before audio starts)
   and reuse on the audio path.
2. **No locks.** No `Mutex::lock`, no `RwLock`, no blocking syscalls.
   For cross-thread state publication, use `Arc<AtomicF32>`,
   `Arc<AtomicU32>`, or a lock-free SPSC ring buffer (`rtrb`,
   `crossbeam-queue::ArrayQueue`).
3. **No logging on the hot path.** No `nih_log!`, no `println!`, no
   `defmt::info!`. If you must trace, write to a lock-free buffer that
   the editor thread drains.

When a module's `[runtime].realtime_audit = true`, codegen emits a
`#[cfg(debug_assertions)]` `assert_no_alloc` guard around `process()`
(via the `assert_no_alloc` crate, declared as an `external_dep` on the
plugin shell module). When `realtime_audit = false`, no guard emits.

## Feature gates

`module.feature_gates[]` lists Cargo features that conditionally
include the module. Emission mirrors library-api:

```rust
// feature_gates = ["midi"]
#[cfg(feature = "midi")]
mod midi_handling;

// feature_gates = ["simd", "fast-math"]
#[cfg(any(feature = "simd", feature = "fast-math"))]
mod dsp_simd;
```

An empty `feature_gates` (or omitted field) means the module is
unconditional. `module_feature_gates_resolve` verifies every name is
declared in `architecture.toml`'s `[[features]]`.

Do **not** combine `feature_gates` with `#[cfg(target_*)]` predicates
in the same `cfg` — emit them as separate `#[cfg(...)]` lines.

## Item emission

`module.items[]` carries the module's contractual surface entries — one
per top-level Rust construct that has spec significance (typically
because it satisfies a requirement). The shape is **prose-first**: each
item carries a `description` of what it does, NOT a signature mirror
of how it's typed. The optional `signature_hint` field exists as an
escape hatch for cases where prose alone is insufficient (FFI
declarations, macro contracts, generics-heavy trait impls); avoid it
for ordinary functions and types.

Sparseness rule: every item except `kind = "re_export"` must carry
`satisfies` (≥1 requirement id). Plugin shell modules typically have
items[] entries for the plugin struct, the parameter struct, and any
exposed-by-bridge widgets; engine and dsp_node modules carry items[]
for their public DSP entry points.

`re_export` items rarely apply on the plugin crate (the cdylib has no
Rust-callable public surface to re-export to). They DO apply on the
engine crate (which uses the library-api profile) when it re-exports
DSP primitives across module boundaries.

See "Skeleton mode (silence-and-log)" below for how skeleton emission
populates audio-thread method bodies — `unimplemented!()` would
abort-crash the host on the first buffer, so the audio-thread surface
gets a different stub variant.

## Cross-module dependencies

`module.depends_on[]` does not directly emit code. Rust's module system
treats `use` statements as the unit of dependency. The `depends_on`
declaration is for the spec's graph checks
(`modules_acyclic` + `layer_dependency`).

When a plugin shell module references the engine, emit
`use crate::<engine_module>::<Item>;` for single-crate layouts, or
`use <engine_crate>::<Item>;` for workspace layouts where the engine
lives in a sibling crate.

## Block and bridge contributions

Module specs in this profile typically carry block-contributed tables:

- `[nih_plug]` block — parameters, audio ports, MIDI, process
  declarative config. The block's `codegen/rust/README.md` drives
  `#[derive(Params)]` emission, range constructors, smoother
  attachment, and the export-macro selection.
- `[egui]` block — editor topology (panels, regions). The block's
  prose drives `nih_plug_egui::create_egui_editor` skeleton emission
  and per-region scaffolding.
- `bindings."egui+nih-plug"` bridge — parameter→widget bindings. The
  bridge's prose drives per-binding widget emission inside region
  loops.

Read the relevant block / bridge READMEs for those emission rules.
This profile prose covers only the architecture-level shape (crate
type, layer ordering, format/category constants, realtime discipline).

## Skeleton mode (silence-and-log)

`gen --skeleton` is supported on this profile via a silence-and-log
stub variant for audio-thread methods. A skeleton-built audio plugin
loads in a host (VST3 / CLAP / standalone) and processes silence for
the lifetime of the session, emitting one diagnostic log line per
audio-path method on its first invocation. Non-audio-path code
(`initialize`, `reset`, editor / state plumbing, hand-authored helpers
that aren't on the audio thread) keeps `unimplemented!()` — these are
called on the main thread and an early panic surfaces missing
implementations during development without risking a host crash.

The split is the load-bearing invariant: `unimplemented!()` on the
audio thread aborts the host. Silence-and-log on the audio thread
keeps the host alive long enough to verify the bundle loads and the
stub surface is wired up correctly.

### Audio-path classification

Two surfaces compose. Neither is sufficient alone.

**Auto-classified by trait method.** The following nih-plug `Plugin`
trait methods are audio-thread by definition; emit silence-and-log
bodies for any of them that show up in the plugin shell module's
emitted impl block:

- `process`
- `process_buffers`
- `task_executor`'s emitted closure body, when the closure is invoked
  on the audio thread (most plugins return a no-op closure here, in
  which case the closure body itself is silence-and-log)

The list is intentionally short — nih-plug's `Plugin` trait keeps
audio-thread method signatures in a tight cluster. New audio-path
methods that nih-plug adds in future revisions extend this list when
the framework dependency bumps. AU support, when authored, supplies
its own audio-path method list in a sibling section here.

**Author opt-in via `is_audio_path`.** Hand-authored DSP-node helpers
(filter step functions, oscillator render fns, voice render entry
points) are not reachable from the trait list. Author-classify them
by setting `is_audio_path = true` on the items[] entry. Skeleton
emission then writes a silence-and-log body for that item exactly the
same way it does for trait methods.

```toml
[[items]]
kind = "fn"
name = "render_block"
description = "Render one block of samples into the output buffer."
satisfies = ["REQ-DSP-1"]
is_audio_path = true
```

Items without `is_audio_path` (the default) emit `unimplemented!()`,
which is the right call for non-audio-path code: the panic surfaces
the gap loudly during normal authoring iteration.

### Silence-and-log emit template

For each audio-path method, the body is:

```rust
fn process(
    &mut self,
    buffer: &mut Buffer,
    _aux: &mut AuxiliaryBuffers,
    _context: &mut impl ProcessContext<Self>,
) -> ProcessStatus {
    #[cfg(debug_assertions)]
    {
        static FIRED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if FIRED
            .swap(true, std::sync::atomic::Ordering::Acquire)
            == false
        {
            nih_log!("[skeleton] process() reached — silence-and-log stub");
        }
    }
    for channel_samples in buffer.iter_samples() {
        for sample in channel_samples {
            *sample = 0.0;
        }
    }
    ProcessStatus::Normal
}
```

The mechanical pieces:

1. **Log surface is `nih_log!`** — the plugin crate already depends on
   nih-plug, so `nih_log!` carries no added Cargo surface. Do not
   reach for `log::info!`, `println!`, or `eprintln!` — those would
   force every skeleton to drag in `env_logger` or emit nothing.

2. **Once-only via `AtomicBool::swap`** — per-call logging would flood
   the host log within a buffer's worth of frames at any meaningful
   sample rate. The static `FIRED` flag flips on the first call and
   subsequent calls take the relaxed-load fast path (the `swap`
   returns true immediately).

3. **Buffer write is silence** — write `0.0` into every sample of
   every channel of `buffer`. For methods that take an output buffer
   not named `buffer` (rare in nih-plug today), substitute the
   parameter name. The point is that the bundle produces audible
   silence rather than uninitialized data.

4. **Non-buffer return values default to `Default::default()`**.
   Audio-path methods returning non-`()` types (e.g. a status enum,
   a `bool`, a numeric tail length) return their `Default` value as
   the universal stub. Authors whose audio-path methods return a
   non-`Default` type must opt out via `is_audio_path = false` (which
   restores `unimplemented!()`) or hand-author the body.

5. **`runtime.realtime_audit = true` gates the once-only log on
   `cfg!(debug_assertions)`.** A user who has opted into the
   no-allocation / no-logging audit on the audio thread has explicitly
   forbidden release-build logging there. The `#[cfg(debug_assertions)]`
   wrapper around the whole logging block keeps the diagnostic loud
   during local development and silent in release-built bundles. The
   `AtomicBool` machinery itself is inside that `cfg`, so realtime-audit
   release builds carry zero stub overhead beyond the empty function
   body and the silence write.

   When `runtime.realtime_audit = false` (the default), the
   `#[cfg(debug_assertions)]` wrapper is still applied — the goal is
   "loud in dev, silent in release" universally. The `realtime_audit`
   flag changes nothing at the silence-and-log site itself; it only
   adds the existing `assert_no_alloc` guard around `process()`, which
   sees the silence-and-log body's lack of allocation just fine.

### `unimplemented!()` retention list

The following methods / surfaces always emit `unimplemented!()` in
skeleton mode, regardless of trait membership or `is_audio_path`:

- `initialize` — runs on the main thread before audio starts. Panic
  here surfaces the gap loudly in the host's plugin-load dialog.
- `reset` — runs on the main thread between transport stops/starts.
  Same reasoning.
- `editor` — returns `Option<Box<dyn Editor>>`; skeleton returns
  `None` rather than panic-on-call (an editor stub that panics would
  crash the host on UI open). Codegen emits `Some(Box::new(no_op_editor))`
  only when the egui block is on the module; otherwise the body is
  `None`.
- `extra_state` / `load_state` — preset / state plumbing on the main
  thread. `unimplemented!()` surfaces the gap when the host attempts
  preset save/restore.
- Any items[] entry with `is_audio_path = false` (or absent — that's
  the default).

The mechanical rule: only audio-thread code gets silence-and-log.
Everything else takes `unimplemented!()` and lets the panic surface
the missing implementation during development. The audio thread is
the one place a panic is strictly unrecoverable; everywhere else a
panic is the fastest gap-finder available.

### What skeleton mode does NOT do

- **Metering / state-publishing writes.** Real plugins publish meter
  values, scope frames, and other audio-thread state to the editor
  via `AtomicCell` writes or lock-free ring buffers. Silence-and-log
  writes nothing back to the editor. The consumer side (the editor)
  is also a stub at this stage, so there's no observable benefit to
  emitting them; this is a bounded follow-up that lands when a real
  plugin needs metering-aware skeleton stubs.
- **MIDI emission.** `process_buffers`'s `context.send_event(...)`
  surface is silent in skeleton mode. Plugins whose primary value is
  generated MIDI (sequencers, arps) will need to hand-author the
  emission or wait for the metering follow-up.
- **AU format support.** Standalone / VST3 / CLAP only in v1; AU
  ships its own audio-thread surface that this section will extend
  when AU support lands.
