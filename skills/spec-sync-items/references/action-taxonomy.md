# `items propose` action taxonomy

For each `pending` entry in `specs/specdev/upstream/items-sync/<Module>.proposal.yaml`, pick exactly one of four actions. Then write `rationale` (one or two sentences naming WHY — refer to the specific requirement, signature mismatch, or sparseness call) and, where required, `proposed_spec`.

## `skip` — leave both spec and source as-is

Pick when the drift represents a deliberate gap.

Common cases:
- `in_source_only` for a private helper the spec deliberately doesn't enumerate. `[[items]]` is a sparse subset of source, not a full mirror — sparseness is intentional.
- `signature_drift` where the spec's `signature_hint` was a documentation aid that's drifted, and the user prefers to drop the hint over chasing source. (`update-spec` with the hint removed is also a valid choice — pick whichever the entry's prose suggests.)

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
  signature_hint: "fn foo(x: u32) -> Result<()>"   # optional
```

## `remove-from-spec` — the spec entry is stale; source has moved on

Pick when the function was renamed or deleted in source and the spec wasn't updated.

`proposed_spec` — absent.

## `update-spec` — the spec entry exists but its shape needs to change

Pick most commonly for `signature_drift` entries where the source signature is the right one and the spec hint is stale.

`proposed_spec` — required. **`update-spec` REPLACES the matched entry; partial merges are not supported.** Carry the full intended shape, not just the changed fields.

## After resolving every entry

- Flip the top-level `status` from `needs-rationale` to `awaiting-approval`.
- **Do NOT set `approved`, `approver`, or `approved_at`.** That's the human's job (Stage 4).
