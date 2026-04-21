# egui — spec-init extensions

This file is loaded by `/spec-init` after the user selects the `egui` domain. It declares the domain-specific portion of project scaffolding.

## Extra interview questions

None. `mounts` and `navigation` are populated from view specs by `/spec-design`, not from an up-front interview.

## architecture.toml skeleton

Emit an empty file. Both `mounts` and `navigation` are optional; entries arrive when root views are declared.

## Extra scaffold directories

- `specs/stores/` — one TOML per shared state container. Write a `.gitkeep` so the empty directory survives git.

## Extra scaffold files

None.

## Post-write notes

Store TOMLs and root-view module TOMLs are authored by `/spec-design`. Panel mounts and the navigation FSM land in `architecture.toml` at that point.
