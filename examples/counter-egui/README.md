# counter-egui

A minimal native Rust GUI example using the [egui](https://github.com/emilk/egui) immediate-mode framework. One screen, two buttons (+ / −), one label, one persisted counter.

Demonstrates the specdev workflow on a non-firmware domain: the same `/spec-design`, `/spec-validate`, `/spec-gen` commands work here, just with `domain = "egui"` in `spec.toml` instead of `"firmware"`.

## Layout

```
counter-egui/
├── spec.toml                      # domain = "egui", language = "rust"
└── specs/
    ├── architecture.toml          # mounts CounterView in central panel
    ├── modules/
    │   └── CounterView.toml       # the view
    ├── stores/
    │   └── CounterStore.toml      # the persisted counter
    └── requirements/
        └── app.toml               # REQ-COUNTER-001, -002
```

## Run the analyzer

From the repo root:

```
specdev-analyzer validate --project specdev/examples/counter-egui
specdev-analyzer extract  --project specdev/examples/counter-egui --output /tmp/counter-out
```

Expected: clean validation, four YAML files written (`CounterView.yaml`, `architecture.yaml`, `requirements.yaml`, `stores/CounterStore.yaml`) with keys alphabetized for byte-stable diffs.

## Regenerate code

Inside Claude Code with the plugin installed:

```
/spec-gen --all
```

This emits Rust modules under `src/` following the egui codegen guides shipped with the plugin (view functions, store wiring, app init).
