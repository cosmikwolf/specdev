# elf-dwarf+probe-rs — Rust codegen conventions

These conventions describe how a `bindings."elf-dwarf+probe-rs"` table on
a module spec maps to Rust source. They layer on top of the project's
profile's `module.conventions.md`, and on top of both endpoint blocks'
own codegen prose.

**Bridge prose wins.** Where this file describes how a sampler's address
list is built, it replaces the `probe-rs` block's instruction to take
addresses as a runtime input: under this binding, the addresses come
from the selection.

## Crate dependencies

This bridge contributes **no new crate**. probe-rs already accepts a
`u64` address and elf-dwarf already produces one, so the pairing needs no
adapter — there is no join-product crate to own. Both blocks' deps.yaml
supply everything the emitted code needs.

What the pairing does contribute is **code**: the mapping from a
recovered DWARF type to a probe access width, which is the one decision
neither block can make alone.

## The handoff, in order

Emit three stages, in this order. They are separate on purpose: the
first two depend on the image and the third depends on the target, and
conflating them is what makes a viewer read stale addresses after a
rebuild.

1. **Recover** — run the selection over the loaded image (elf-dwarf).
2. **Plan** — map each recovered variable to an access (this bridge).
3. **Read** — execute the plan against the running core (probe-rs).

## Plan emission — the type-to-width mapping

This is what the bridge exists for. Emit width **and** interpretation
together:

```rust
ScalarKind::Bool => (Access::Word8, Interpret::Bool),
ScalarKind::Float => match recovered.byte_size {
    4 => (Access::Word32, Interpret::Float),
    8 => (Access::Word64, Interpret::Float),
    _ => return None,
},
ScalarKind::Signed | ScalarKind::Unsigned => {
    let access = match recovered.byte_size {
        1 => Access::Word8,
        2 => Access::Word16,
        4 => Access::Word32,
        8 => Access::Word64,
        _ => return None,
    };
    ...
}
```

A `u32` and an `f32` are the same four-byte read and different numbers,
so a plan recording only the width would throw away exactly the half of
the type the debug info was consulted for.

A recorded size that is not 1, 2, 4 or 8 is **refused**, not rounded up.
Rounding reads neighbouring bytes and presents them as part of the
value — a wrong number that looks like a right one.

`width_from = "sampler_default"` uses the sampler's `default_width` for
every access but still takes the interpretation from the type. The
sampler has an opinion about transfer width, not about whether the bytes
are a float.

`aggregates` decides what happens to a `ScalarKind::Other`:

```rust
ScalarKind::Other => match aggregates {
    Aggregates::Skip => return None,
    Aggregates::Bytes => (
        Access::Bytes(usize::try_from(recovered.byte_size).ok()?),
        Interpret::Raw,
    ),
},
```

`Aggregates::Bytes` emits **one** transaction, never a series of words:
a struct read as four separate words can tear across a write from the
running target.

Reading a value back needs the interpretation again, and sign extension
is the reason it is carried at all — a `-1i16` arrives as `0xFFFF`, and
widening without extending gives 65535:

```rust
Interpret::Signed => match access {
    Access::Word8 => (raw as u8 as i8) as f64,
    Access::Word16 => (raw as u16 as i16) as f64,
    Access::Word32 => (raw as u32 as i32) as f64,
    Access::Word64 => (raw as i64) as f64,
    Access::Bytes(_) => raw as f64,
},
Interpret::Float => match access {
    Access::Word32 => f32::from_bits(raw as u32) as f64,
    Access::Word64 => f64::from_bits(raw),
    _ => raw as f64,
},
```

Classify from gimli's own constants rather than a transcribed integer
table:

```rust
pub fn kind_from_encoding(encoding: gimli::DwAte) -> ScalarKind {
    match encoding {
        gimli::DW_ATE_boolean => ScalarKind::Bool,
        gimli::DW_ATE_float => ScalarKind::Float,
        gimli::DW_ATE_signed | gimli::DW_ATE_signed_char => ScalarKind::Signed,
        gimli::DW_ATE_unsigned | gimli::DW_ATE_unsigned_char => ScalarKind::Unsigned,
        _ => ScalarKind::Other,
    }
}
```

**Do NOT** emit `f32::from_bits` for a float of any size other than 4,
or `f64::from_bits` for any size other than 8. **Do NOT** drop a
refused variable silently — surface it, because "my variable is missing"
with no explanation is the hardest state to debug in a viewer. **Do
NOT** widen a signed value before sign-extending it.

## Read emission

Once planned, every read is an ordinary probe-rs read; the bridge's
contribution already happened. Dispatch on the plan's `Access`, honour
the sampler's `max_batch` and `on_error` exactly as the probe-rs block's
prose describes:

```rust
let mut core = session.core(core_index)?;

for chunk in plans.chunks(max_batch) {
    for item in chunk {
        out.push(read_one(&mut core, item).ok());
    }
}
```

`on_missing` is checked at plan time, not read time:

```rust
if plans.is_empty() && on_missing == OnMissing::Error {
    return Err("selection yielded no samplable variables".into());
}
```

**Do NOT** re-run the selection inside the sampling loop. Walking DWARF
costs orders of magnitude more than a memory read, and the answer only
changes when the image does.

## Reload emission — the case this pairing exists for

When the bound selection's image has `reload = "watch"`, emit a
re-plan on reload, and **rebuild the plan wholesale**:

```rust
let rebuilt = plan(recovered, aggregates, on_missing)?;
*existing = rebuilt;
```

Addresses move on every rebuild, so a plan outlives its image only by
accident. Patching it is wrong rather than merely inefficient: a
variable can change size or disappear, not just move, and a patched plan
would keep reading the old width at the new address — producing numbers
the whole way, all of them wrong.

**Do NOT** keep sampling from the old plan while the new image loads.
Stop the pass, re-plan, restart. **Do NOT** assume a variable that kept
its name kept its type.

## Discipline rules

These are cross-cutting constraints that no single block can check, and
that the reduction cannot express as a relation. Emitted code enforces
them; a future graph-check primitive may promote them.

- **EDPR-001.** A selection whose `kinds` admits only `other`, bound with
  `aggregates = "skip"`, always yields an empty address list. Reject at
  codegen time with a clear message rather than emitting a sampler that
  silently reads nothing.
- **EDPR-002.** A selection bound to more than one sampler is allowed —
  the same symbols may legitimately be read at two rates — but each
  binding plans independently. Do not share a plan between samplers.
- **EDPR-003.** The image and the target must be the same build. Nothing
  in either block can verify this, and the failure mode is reading
  plausible values from wrong addresses. Where the target exposes a build
  identifier, emit a check against it; where it does not, say so in the
  generated documentation rather than implying a guarantee.
