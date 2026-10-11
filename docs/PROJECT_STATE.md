# Cellworld Project Handoff

This page summarizes the current project so work can resume from a fresh clone
or a different VS Code computer. The source of truth is this Git repository;
session chat history is not required.

## Project

Cellworld is a Windows-first Rust 2024 application that simulates abstract
cell populations competing for feed in a 3D integer-coordinate field. It uses
Bevy for its interactive 3D viewer and keeps simulation rules in a Rust library.
The initial cell kind is `BasicCellType`; additional kinds implement the
`CellType` trait.

## Current structure

- `src/lib.rs`: public simulation API and module exports.
- `src/field.rs`: 3D integer coordinates.
- `src/cells.rs`: shared cell state and type-specific rule interface.
- `src/feed.rs`: sparse feed storage.
- `src/signals.rs`: fast local and slow wider-area signal calculations.
- `src/world.rs`: movement, feed uptake, energy, reproduction, and world steps.
- `src/main.rs`: application entry point.
- `src/viewer.rs`: Bevy app, rendering, HUD, mouse camera, and fixed-step loop.
- `tests/simulation.rs`: deterministic integration coverage of simulation
  rules and custom cell types.
- `.github/copilot-instructions.md`: project-wide assistant guidance.
- `.github/agents/`: Cellworld architecture, cell, field, and test specialists.
- `docs/DECISIONS.md`: canonical record of accepted architectural and behavior
  decisions.

## Run and validate

Install a stable Rust toolchain with the MSVC Windows target and a working
graphics driver, clone the repository, then run:

```powershell
cargo run
```

Optional initial feed generation:

```powershell
cargo run -- 0.5
```

Validation:

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

The viewer opens a window. Space pauses/resumes; Escape closes it. The top-left
HUD's −/+ buttons adjust feed generated per tick by 0.1. Left-drag orbits,
right-drag pans, and the scroll wheel zooms. Simulation steps occur every
0.5 seconds and stop when no cells remain.

## Continuing work

1. Read this page, `.github/copilot-instructions.md`, and
   `docs/DECISIONS.md` before changing project direction.
2. Implement simulation behavior outside the Bevy viewer where practical.
3. Add deterministic tests for new simulation rules and edge cases.
4. Record every accepted architectural, simulation, and user-facing behavior
   decision in `docs/DECISIONS.md` in the same change that implements it.
   Explicitly supersede earlier decisions rather than silently replacing them.
5. Update this handoff page or the README when project structure, setup, or
   user-facing controls change.

## Not implemented yet

There is no persistence/save-load, networking, evolution system, task/reward
system, in-viewer restart, or UI to add/remove cells. Do not assume these
features or their design decisions exist.
