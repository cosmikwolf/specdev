# `items propose` action taxonomy

For each `pending` entry in `specs/specdev/upstream/items-sync/<Module>.proposal.yaml`, pick exactly one of four actions. Then write `rationale` (one or two sentences naming WHY — refer to the specific requirement, rename, or sparseness call) and, where required, `proposed_spec` or `rename_to`.

## `skip` — leave both spec and source as-is

Pick when the drift represents a deliberate gap.

Common cases:
- `in_source_only` for a private helper the spec deliberately doesn't enumerate. `[[items]]` is a sparse subset of source, not a full mirror — sparseness is intentional.

`proposed_spec` — absent.

## `add-to-spec` — enumerate this source item in the spec

Pick when the source item is contractual and traces to a requirement.

Common case: `in_source_only` entries where the function/struct/etc. is part of the module's public surface AND traces to a requirement.

`proposed_spec` — required. Carry the full new `[[items]]` mapping:

```yaml
proposed_spec:
  kind: fn          # or struct / enum / trait / re_export / ...
  name: foo
  description: One-line summary.
  satisfies: [REQ-001]   # required for every kind except re_export (sparseness invariant)
```

## `remove-from-spec` — the spec entry is stale; source deleted it

Pick when the item was **deleted** in source and the spec wasn't updated — the requirement it satisfied is genuinely gone.

If the item was **renamed** rather than deleted, use `rename` instead (below) — `remove-from-spec` followed by `add-to-spec` would drop the `satisfies` trace and force you to re-author it by hand.

`proposed_spec` — absent.

## `rename` — the item was renamed in source; carry its trace across

Pick when a top-level item was renamed in source: the old name shows up as `in_spec_only` and the new name as a separate `in_source_only` entry. A rename rewrites only the spec item's `name`, preserving its `satisfies`, `description`, and kind-discriminated extras — the whole point is keeping the requirement trace that remove + add would lose.

Set the action on the **`in_spec_only` entry** (the one carrying the old name and the trace):

```yaml
recommendation:
  action: rename
  rename_to: apply_refill      # the new name now in source
  rationale: "Renamed refill -> apply_refill in code; preserving the REQ-007 trace."
```

Then mark the paired **`in_source_only` entry** (the new name) `skip` — it's accounted for by the rename, and the next `items diff` will pair the renamed spec item with the source symbol (no durable skip needed).

`proposed_spec` — absent; use `rename_to` instead.

## After resolving every entry

- Flip the top-level `status` from `needs-rationale` to `awaiting-approval`.
- **Do NOT set `approved`, `approver`, or `approved_at`.** That's the human's job (Stage 4).
