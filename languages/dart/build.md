# Dart Build & Toolchain Patterns

Pure toolchain reference for Dart / Flutter projects. These patterns apply to any domain.

---

## pubspec.yaml

The package manifest. Minimal library package:

```yaml
name: my_package
description: One-line package description.
version: 0.1.0
environment:
  sdk: ^3.6.0

dependencies:
  collection: ^1.18.0

dev_dependencies:
  lints: ^5.0.0
  test: ^1.25.0
```

A Flutter app adds the SDK dependency and the `flutter:` section:

```yaml
environment:
  sdk: ^3.6.0
  flutter: ">=3.27.0"

dependencies:
  flutter:
    sdk: flutter

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^5.0.0

flutter:
  uses-material-design: true
```

### Dependency shapes

```yaml
dependencies:
  hosted_default: ^1.2.3            # pub.dev, caret constraint (the default)
  hosted_range: ">=1.2.0 <2.0.0"    # explicit range
  custom_host:                       # non-default package repository
    hosted: https://pub.example.com
    version: ^0.4.0
  from_git:
    git:
      url: https://github.com/acme/from_git.git
      ref: v1.4.0                    # tag / branch / commit (pin for reproducibility)
      path: packages/from_git        # when the package is not at the repo root
  local_sibling:
    path: ../local_sibling           # path dep; no version key
  flutter:
    sdk: flutter                     # SDK-provided package
```

- `dev_dependencies` — test/lint/codegen tooling; never shipped.
- `dependency_overrides` — local-development escape hatch only; do not commit
  overrides that generated manifests would fight over.

---

## Pub Workspaces

Multi-package repos use pub workspaces (Dart ≥3.6): one shared resolution and
a single lockfile at the root.

Root `pubspec.yaml`:

```yaml
name: my_workspace
environment:
  sdk: ^3.6.0
workspace:
  - packages/app
  - packages/core
```

Each member `pubspec.yaml` adds:

```yaml
resolution: workspace
```

Run `dart pub get` once at the root; members resolve together. (Melos ≥7
delegates dependency resolution to pub workspaces and remains useful purely as
a task runner.)

---

## Toolchain Commands

| Purpose | Pure Dart package | Flutter package/app |
|---|---|---|
| Fetch deps | `dart pub get` | `flutter pub get` |
| Static analysis (the compile gate) | `dart analyze` | `flutter analyze` |
| Format | `dart format .` | `dart format .` |
| Run tests | `dart test` | `flutter test` |
| Run the app | `dart run` | `flutter run -d <device>` |
| Build release | `dart compile exe` | `flutter build apk` / `appbundle` / `ios` / `web` |

- `dart analyze` / `flutter analyze` exiting 0 is the Dart analog of
  `cargo check` — an analysis-clean tree is the minimum bar for generated code.
- `flutter build web` targets JS by default; `flutter build web --wasm` targets
  WasmGC and is stricter about interop — treat it as a separate gate.

---

## analysis_options.yaml

```yaml
include: package:lints/recommended.yaml   # or package:flutter_lints/flutter.yaml
analyzer:
  language:
    strict-casts: true
    strict-inference: true
    strict-raw-types: true
```

Generated code must be clean under the package's committed analysis options —
do not weaken the options to make generated code pass.

---

## Flutter Platform Directories

`flutter create` scaffolds `android/`, `ios/`, `web/`, `macos/`, `linux/`,
`windows/`. These are build-system state, owned by the Flutter tool and edited
only for platform configuration (permissions, entitlements, manifest entries).
Spec tooling never generates or records them.

Platform permission declarations live in:

- Android: `android/app/src/main/AndroidManifest.xml`
- iOS: `ios/Runner/Info.plist`
- Web: no manifest — capabilities are browser-mediated at runtime

---

## Codegen Runners

Some packages generate Dart source (`*.g.dart`, `*.freezed.dart`) via
`build_runner`; others ship standalone generator CLIs. Two rules:

1. Generated files are **externally owned** — committed or ignored per that
   tool's convention, but never treated as hand-authored source.
2. Generator invocation order matters when output of one feeds another; the
   consuming project documents the sequence (e.g. run the FFI binding generator
   before `flutter build`).
