#!/usr/bin/env bash
# Phase 31-B — build every codegen reference crate.
#
# A block/bridge may ship `reference/<lang>/` — a CI-only crate that type-checks
# the worked examples in its `codegen/<lang>/README.md` against the pinned
# framework API. A wrong generic arity, constructor arg order, unbound interrupt,
# or raw-Peri-vs-handle mistake in the prose fails the compile here, before it
# can reach a downstream `/spec-gen` dogfood. These crates are NOT inlined into
# user projects (they are siblings to `codegen/`, which inline never copies from).
#
# Each reference crate carries its own `.cargo/config.toml` pinning the target
# triple, so a plain `cargo build` cross-compiles. Run from anywhere.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"   # plugin root (specdev/)
fail=0
found=0

while IFS= read -r manifest; do
  found=$((found + 1))
  dir="$(dirname "$manifest")"
  rel="${dir#"$ROOT"/}"
  echo "==> building reference crate: $rel"
  if ( cd "$dir" && cargo build --quiet ); then
    echo "    OK"
  else
    echo "    FAIL: $rel"
    fail=1
  fi
done < <(find "$ROOT/blocks" "$ROOT/bridges" -path '*/reference/*/Cargo.toml' 2>/dev/null | sort)

echo "----"
echo "reference crates built: $found, failures: $fail"
[ "$found" -eq 0 ] && echo "(no reference crates found yet — Phase 31-B backfill pending)"
exit "$fail"
