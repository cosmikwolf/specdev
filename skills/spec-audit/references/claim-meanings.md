# What each audited field claims

The meanings for the fields and values the plugin ships. A `description` in the project's inlined schema wins over
this file: the project may have narrowed a meaning. These fields are declared by `firmware-layered`; the same rules
apply to any profile that declares a field with the same name.

"The action" below means every path in the action's code, including this module's helpers it calls and the code it
calls in other project modules (sibling workspace members included). Stop at code outside the repository and trust
its documented behaviour. Another module's spec flags are a hint only: judge by its code.

## Action flags

| Field | Claims | `contradicted` by | Typical evidence for `holds` |
|---|---|---|---|
| `blocking = false` | The action never holds its caller waiting. Awaiting a future that yields is not a wait: the executor runs other work, whatever the callee's spec says. | A busy-wait or polling loop with no fixed small bound (also inside an awaited project `async fn`), a sleep or delay call that does not yield, a blocking channel or lock call, a synchronous call whose code waits. | The waits are awaits of futures that yield, and the loops have a fixed small bound. |
| `isr_safe = true` | The action may run in an interrupt handler: it does not block, does not allocate, and takes no lock that the interrupted code can hold without the interrupt masked. | Any `blocking = false` contradiction above (an `.await` too: an interrupt handler cannot await); a heap allocation; a mutex or spin lock that thread code takes with interrupts on; a call into code that is not interrupt-safe. | The body touches shared state only inside a critical section or through atomics, and calls only interrupt-safe code. |

`blocking = true` and `isr_safe = false` allow; they promise nothing and are not audited. A long critical section is
not a wait, so it does not contradict `blocking = false`; when one masks interrupts for long, say so under "Other
findings".

## Action parameters

| Field | Claims | `contradicted` by |
|---|---|---|
| `params[].name`, `params[].direction` | The action's code takes a parameter with each name, used in each direction: `in` (read: by value — a `mut` binding of an owned value included — or `&`), `out` (written: `&mut`, not read first), `inout` (read and written: `&mut`). One claim per action. | A name the signature does not have, a signature parameter the spec does not list, or a direction the type or the body contradicts. |

## Unit-table entries

| Field | Claims | `contradicted` by |
|---|---|---|
| one `[[<unit>.<table>]]` entry | The code has and uses the thing the entry names, with each field's value, as the unit's schema and guidance define the table (for example: a DMA channel the module takes at init, wired to the named source). | No code that takes or uses it (cite the search), or a field value the code does not match. |

## Metadata categories

| Field | Claims | `contradicted` by |
|---|---|---|
| `MemoryModel.allocation = "static_alloc"` | The module's state lives in the struct or in statics; no heap. | A heap type in the module's own code: `Box`, `alloc::vec::Vec` / `std::vec::Vec`, `String`, `Rc` / `Arc`, any `alloc::` path. Fixed-capacity types (`heapless::Vec`, arrays) are not heap. |
| `MemoryModel.allocation = "stack_alloc"` | State is built on the stack per call; no heap, no long-lived statics of its own. | A heap type, or a `static mut` / `static` cell the module owns. |
| `MemoryModel.allocation = "heap_alloc"` | Owned heap data is allowed. | Nothing — it allows. |
| `MemoryModel.max_instances = N` | At most N instances exist. | A constructor reachable more than N times (a loop, a pool larger than N). Usually `not shown`. |
| `MemoryModel.owner`, `Ownership.owner` | Who owns **every** buffer and resource the module handles — its parameters, its fields, and the statics it touches. `owned`: the module holds each by value, and no other module touches it. `borrowed`: it holds references to things others own. `shared`: others touch them too. | `owned` when any buffer or static the module touches is also touched by another module (name both places), or is held by reference. `borrowed` when it holds a resource by value. |
| `Ownership.borrow_lifetime` | The lifetime the borrowed resources have. | A field or parameter lifetime that differs. |
| `ConcurrencyModel.protection` | The one guard for all the module's shared mutable state: `none`, `critical_section`, `mutex`, `atomic`, `disable_interrupts`. | An access to shared mutable state outside the named guard, including state guarded by another means (atomics, a hand-off protocol). `none` is contradicted by any shared mutable state. When the code uses a mix, hand off to `/spec-question`. |
| `ConcurrencyModel.shared_state = [...]` | These are the shared items the module's own code touches directly: every static holding data (atomics included) that another module or an interrupt also touches, whatever guards it. Channels, signals and queues are message passing, not shared state; a static reached only through a neighbour's function is the neighbour's. Diagnostic counters count too. | A listed name no code touches (search the repository's source and test roots), or a shared static the module touches that the list lacks. Name each one. |
| `ConcurrencyModel.isr_safe` | The module as a whole is safe to call from an interrupt; as the action flag, for every action. | As the action flag. |
| `ConcurrencyModel.reentrant = true` | Two calls may overlap safely. | A shared mutable static touched without a guard. |
| `ErrorHandling.strategy` | How the module's actions report failure: `result_type` (a `Result`), `enum_variant` (an error variant in a returned or sent value), `option`, `errno` (an integer code), `panic` (failure stops the program). | A failure reported another way. For `panic`: a failure that returns, is sent as a value, or is logged and dropped. For the others: an `unwrap`, `expect` or `panic!` on a recoverable failure, or a failure dropped silently. |
| `ErrorHandling.error_type` | The error type the actions return. | A different error type in the signatures. |
| `Timing.worst_case_us`, `typical_us`, `deadline_us` | Time budgets. | Code cannot settle these: `not shown`, unless a measurement or a timing test is in the tree. Name what would settle it. |
