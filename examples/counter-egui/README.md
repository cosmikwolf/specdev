# counter-egui — minimal egui domain fixture

The "blinky" of the egui domain: a one-screen native Rust GUI app with
a persisted counter. Two buttons (+ / −), one label, one store.

Proves the specdev analyzer is domain-neutral by exercising the full
validate → extract pipeline against a non-firmware project.

## Layout

```
counter-egui/
├── spec.toml                            # project config (domain = "egui")
└── specs/
    ├── architecture.toml                # mounts CounterView in central
    ├── modules/
    │   └── CounterView.toml             # the one view
    ├── stores/
    │   └── CounterStore.toml            # the one store (persisted)
    └── requirements/
        └── app.toml                     # REQ-COUNTER-001, -002
```

## Smoke

```
specdev-analyzer validate specdev/examples/counter-egui
specdev-analyzer extract -o /tmp/counter-egui-out specdev/examples/counter-egui
```

Expected: zero errors; extraction produces `CounterView.yaml`,
`architecture.yaml`, `requirements.yaml`, and `stores/CounterStore.yaml`
with keys alphabetically sorted.
