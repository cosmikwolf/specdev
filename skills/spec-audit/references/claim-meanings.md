# What each audited field claims

The meanings for the fields the plugin ships. A `description` in the project's inlined schema wins over this file:
the project may have narrowed a meaning. These fields are declared by `firmware-layered`; the same rules apply to any
profile that declares a field with the same name.

"The action" below means every path the action's code can take, including calls into this module's helpers. A call
into another project module counts through what that module's own spec claims, unless its code is in front of you.

## Action flags

| Field | Claims | `contradicted` by | Typical evidence for `holds` |
|---|---|---|---|
| `blocking = false` | The action never holds its caller waiting. An `.await` on a future is not blocking — the executor runs other work. | A busy-wait or polling loop on hardware state, a sleep or delay call that does not yield, a blocking channel or lock call, a synchronous call into a module whose action is `blocking = true`. | The waits are `.await`s, or the loops have a fixed small bound. |
| `isr_safe = true` | The action may run in an interrupt handler: it does not block, does not allocate, and takes no lock that the interrupted code can hold without the interrupt masked. | Any `blocking = false` contradiction above; a heap allocation; a mutex or spin lock that thread code takes with interrupts on; a call into an action that is `isr_safe = false`. | The body touches shared state only inside a critical section or through atomics, and calls only `isr_safe` code. |

`blocking = true` and `isr_safe = false` allow; they promise nothing and are not audited.

## Action parameters

| Field | Claims | `contradicted` by |
|---|---|---|
| `params[].name` | The action's code takes a parameter with this name. | A name the signature does not have, or a signature parameter the spec does not list. |

## Metadata categories

| Field | Claims | `contradicted` by |
|---|---|---|
| `MemoryModel.allocation = "static_alloc"` | The module's state lives in the struct or in statics; no heap. | `Box`, `Vec`, `String`, `Rc` / `Arc`, `alloc::` in the module's own code. |
| `MemoryModel.allocation = "stack_alloc"` | State is built on the stack per call; no heap, no long-lived statics of its own. | A heap type, or a `static mut` / `static` cell the module owns. |
| `MemoryModel.allocation = "heap_alloc"` | Owned heap data is allowed. | Nothing — it allows. |
| `MemoryModel.max_instances = N` | At most N instances exist. | A constructor reachable more than N times (a loop, a pool larger than N). Usually `not shown`. |
| `MemoryModel.owner`, `Ownership.owner` | `owned`: the module owns its buffers and resources. `borrowed`: it holds references it does not own. `shared`: others hold them too. | A field type that disagrees: `&'a T` / `&'static T` for `owned`; an owned field for `borrowed`. |
| `Ownership.borrow_lifetime` | The lifetime the borrowed resources have. | A field or parameter lifetime that differs. |
| `ConcurrencyModel.protection` | How shared state is guarded: `none`, `critical_section`, `mutex`, `atomic`, `disable_interrupts`. | An access to shared state outside the named guard; a different guard. `none` is contradicted by any shared mutable state. |
| `ConcurrencyModel.shared_state = [...]` | These are the shared items the module touches. | A shared static the module touches that is not listed, or a listed one it never touches. Name each one. |
| `ConcurrencyModel.isr_safe` | The module as a whole is safe to call from an interrupt; as the action flag, for every action. | As the action flag. |
| `ConcurrencyModel.reentrant = true` | Two calls may overlap safely. | A shared mutable static touched without a guard. |
| `ErrorHandling.strategy` | How the module's actions report failure: `result_type` (a `Result`), `enum_variant` (an error enum in the return), `option`, `errno` (an integer code), `panic` (failure stops the program). | An action that reports failure another way — a `Result` in a `panic` module is a finding only when the failure is then dropped; an `unwrap` or `panic!` on a recoverable error in a non-`panic` module. |
| `ErrorHandling.error_type` | The error type the actions return. | A different error type in the signatures. |
| `Timing.worst_case_us`, `typical_us`, `deadline_us` | Time budgets. | Code cannot settle these: `not shown`, unless a measurement or a timing test is in the tree. Name what would settle it. |
