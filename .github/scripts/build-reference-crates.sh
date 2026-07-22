#!/usr/bin/env bash
# Build/analyze every codegen reference crate.
#
# A block/bridge may ship `reference/<lang>/` — a CI-only crate that type-checks
# the worked examples in its `codegen/<lang>/README.md` against the pinned
# framework API. A wrong generic arity, constructor arg order, unbound interrupt,
# or raw-Peri-vs-handle mistake in the prose fails the compile here, before it
# can reach a downstream `/spec-gen` dogfood. These crates are NOT inlined into
# user projects (they are siblings to `codegen/`, which inline never copies from).
#
# Rust references: each carries its own `.cargo/config.toml` pinning the target
# triple, so a plain `cargo build` cross-compiles. Dart references: discovered
# by `reference/dart/pubspec.yaml`; the gate is `pub get` + `analyze`, run with
# `flutter` when the pubspec depends on the Flutter SDK, plain `dart` otherwise.
# Run from anywhere.
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

while IFS= read -r manifest; do
  found=$((found + 1))
  dir="$(dirname "$manifest")"
  rel="${dir#"$ROOT"/}"
  tool=dart
  grep -q 'sdk: *flutter' "$manifest" && tool=flutter
  echo "==> analyzing reference package: $rel (via $tool)"
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "    FAIL: $rel ($tool not on PATH — provision the SDK in CI)"
    fail=1
    continue
  fi
  if ( cd "$dir" && "$tool" pub get >/dev/null && "$tool" analyze ); then
    echo "    OK"
  else
    echo "    FAIL: $rel"
    fail=1
  fi
done < <(find "$ROOT/blocks" "$ROOT/bridges" -path '*/reference/dart/pubspec.yaml' 2>/dev/null | sort)

echo "----"
echo "reference crates built: $found, failures: $fail"
[ "$found" -eq 0 ] && echo "(no reference crates found yet)"
exit "$fail"
