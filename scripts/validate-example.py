#!/usr/bin/env python3
"""
validate-example.py — Development harness for the firmware v1 schemas.

Loads every YAML schema under schema/firmware/v1/, registers each by filename
so relative $refs resolve, and then validates each example TOML under
examples/blinky-a2dp/specs/ against the appropriate schema.

This is a phase-2 authoring loop aid; the real validator is specdev-analyzer's
`validate` subcommand (phase 4).

Usage:
    python3 scripts/validate-example.py [--fail-loud PATH] [--fail-on-pass]

Exits 0 iff every expected-valid example validates and every expected-invalid
example fails validation.
"""
from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012

PLUGIN_ROOT = Path(__file__).resolve().parent.parent
SCHEMA_DIR = PLUGIN_ROOT / "schema" / "firmware" / "v1"
EXAMPLES = PLUGIN_ROOT / "examples" / "blinky-a2dp"

# Map example file (relative to EXAMPLES) → schema filename it must satisfy.
VALID_CASES: dict[str, str] = {
    "specs/modules/StatusLed.toml":     "module.schema.yaml",
    "specs/modules/BtA2dpSink.toml":    "module.schema.yaml",
    "specs/modules/I2sOutput.toml":     "module.schema.yaml",
    "specs/modules/AudioPipeline.toml": "module.schema.yaml",
    "specs/architecture.toml":          "architecture.schema.yaml",
    "specs/requirements/audio.toml":    "requirements.schema.yaml",
}


def build_registry() -> tuple[Registry, dict[str, dict]]:
    """Load all YAML schemas and register under filename + $id."""
    schemas: dict[str, dict] = {}
    resources: list[tuple[str, Resource]] = []
    for schema_path in sorted(SCHEMA_DIR.glob("*.schema.yaml")):
        raw = yaml.safe_load(schema_path.read_text())
        schemas[schema_path.name] = raw
        resource = Resource(contents=raw, specification=DRAFT202012)
        # Register under the bare filename (used by local $refs in our files).
        resources.append((schema_path.name, resource))
        # Also register under $id if present.
        sid = raw.get("$id")
        if sid:
            resources.append((sid, resource))
    registry = Registry().with_resources(resources)
    return registry, schemas


def validate_case(schemas: dict, registry: Registry, spec: dict,
                  schema_name: str) -> list[str]:
    schema = schemas[schema_name]
    validator = Draft202012Validator(schema, registry=registry)
    errors = []
    for err in sorted(validator.iter_errors(spec), key=lambda e: list(e.absolute_path)):
        path = "/".join(str(p) for p in err.absolute_path) or "<root>"
        errors.append(f"  at {path}: {err.message}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fail-loud", type=Path,
                        help="Additional spec file expected to FAIL; print its errors.")
    parser.add_argument("--against", default="module.schema.yaml",
                        help="Schema to use with --fail-loud (default: module.schema.yaml).")
    args = parser.parse_args()

    registry, schemas = build_registry()

    # Meta-schema: verify each schema is itself a valid JSON Schema 2020-12 doc.
    meta = Draft202012Validator.META_SCHEMA
    meta_validator = Draft202012Validator(meta)
    for name, schema in schemas.items():
        meta_errors = list(meta_validator.iter_errors(schema))
        if meta_errors:
            print(f"META-SCHEMA FAIL: {name}")
            for e in meta_errors:
                path = "/".join(str(p) for p in e.absolute_path) or "<root>"
                print(f"  at {path}: {e.message}")
            return 1
    print(f"OK: {len(schemas)} schema(s) pass the JSON Schema 2020-12 meta-schema.")

    failures = 0
    for rel, schema_name in VALID_CASES.items():
        path = EXAMPLES / rel
        data = tomllib.loads(path.read_text())
        errors = validate_case(schemas, registry, data, schema_name)
        if errors:
            failures += 1
            print(f"INVALID (expected valid): {rel}")
            for e in errors:
                print(e)
        else:
            print(f"OK:  {rel}  →  {schema_name}")

    if args.fail_loud:
        data = tomllib.loads(args.fail_loud.read_text())
        errors = validate_case(schemas, registry, data, args.against)
        print(f"\n--- {args.fail_loud} against {args.against} ---")
        if errors:
            print(f"FAILED (expected): {len(errors)} error(s)")
            for e in errors:
                print(e)
        else:
            print("PASSED (unexpected) — this spec should have been rejected.")
            failures += 1

    return 0 if failures == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
