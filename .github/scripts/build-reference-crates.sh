#!/usr/bin/env bash
# Build/analyze every reference crate.
#
# A block/bridge may ship `reference/<lang>/` — a CI-only crate written against
# the real framework API. The codegen prose these crates once witnessed is gone
# (specdev no longer generates code); they stay, compiled, as the corpus the
# rule-pack work measures against, and leave the tree with the reference
# lint once that work settles. They are never inlined into user projects.
#
# Rust references: each carries its own `.cargo/config.toml` pinning the target
# triple, so a plain `cargo build` cross-compiles. Dart references: discovered
# by `reference/dart/pubspec.yaml`; the gate is `pub get` + `analyze`, run with
# `flutter` when the pubspec depends on the Flutter SDK, plain `dart` otherwise.
# TypeScript references: discovered by `reference/typescript/package.json`; the
# gate is `pnpm install --frozen-lockfile` + `pnpm run check`.
#
# The TypeScript gate is indirect ON PURPOSE. Rust and Dart have one compile
# command each, so this script can name it. A TypeScript reference may need a
# block-specific step first — the prisma reference must run `prisma generate`
# before `tsc` sees the generated client — and that step belongs to the block,
# not to the language. So each reference declares its own `check` script and
# this loop runs that. A reference whose package.json has no `check` script is
# a failure, not a skip: a missing gate that reads as a pass is the false green
# this tree keeps filing issues about.
#
# `--frozen-lockfile` is deliberate here: this lockfile is committed and its
# staleness is exactly what is under test.
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

while IFS= read -r manifest; do
  found=$((found + 1))
  dir="$(dirname "$manifest")"
  rel="${dir#"$ROOT"/}"
  echo "==> checking reference package: $rel (via pnpm)"
  if ! command -v pnpm >/dev/null 2>&1; then
    echo "    FAIL: $rel (pnpm not on PATH — provision it in CI)"
    fail=1
    continue
  fi
  # A reference with no `check` script has no gate. Say so rather than
  # running nothing and reporting OK.
  if ! grep -q '"check"' "$manifest"; then
    echo "    FAIL: $rel (package.json declares no \`check\` script)"
    fail=1
    continue
  fi
  if ( cd "$dir" && pnpm install --frozen-lockfile >/dev/null && pnpm run check ); then
    echo "    OK"
  else
    echo "    FAIL: $rel"
    fail=1
  fi
done < <(find "$ROOT/blocks" "$ROOT/bridges" -path '*/reference/typescript/package.json' 2>/dev/null | sort)

echo "----"
echo "reference crates built: $found, failures: $fail"
[ "$found" -eq 0 ] && echo "(no reference crates found yet)"
exit "$fail"
