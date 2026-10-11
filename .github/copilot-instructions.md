# Cellworld Project Copilot Instructions

## Project

Cellworld is a Windows-first interactive simulation sandbox written in Rust. It
models generic cells in a three-dimensional field: populations compete for
spatially distributed feed, communicate through distinct fast and slow signal
channels, and may later evolve through inheritance and variation. A 3D viewer is
an important part of the project, not an afterthought.

Cell kinds share common state and implement the `CellType` trait for kind-
specific rules. Keep the initial simulation abstract; biological fidelity is
not a requirement. Evolution and task/reward systems are later work.

The project must use:

- Stable Rust
- Rust 2024 edition
- Cargo for dependency management, building, testing, and running
- rustfmt for formatting
- clippy for static analysis
- cargo test for automated tests

Do not introduce nightly-only Rust features unless the Architect explicitly
approves the decision and documents the reason in docs/DECISIONS.md.

Bevy is the selected engine for the initial desktop viewer. Keep Bevy-specific
presentation, input, and scheduling in the viewer, and keep simulation rules
in the portable library. Propose other significant engine, renderer, spatial
data structure, or simulation changes to the Architect and record accepted
decisions.

## General Engineering Principles

Prioritize:

1. Correctness
2. Maintainability
3. Testability
4. Clear visualization and usability
5. Performance
6. Simplicity

Do not introduce abstractions without a concrete reason.

Prefer small, cohesive modules with clear ownership of responsibilities.

Keep simulation rules independent from rendering, UI, and platform-specific
code. The simulation core should remain portable where practical, even though
the first application target is Windows desktop.

Avoid putting simulation rules directly into rendering code. The viewer should
observe and control the simulation through explicit state, commands, or
snapshots rather than owning the simulation's rules.

Avoid global mutable state unless there is a documented architectural reason.

Prefer explicit data flow over hidden state.

Use Rust ownership and borrowing idiomatically. Do not fight the borrow checker
with unnecessary cloning, reference counting, or interior mutability.

## Cargo

Use Cargo commands rather than manually modifying generated Cargo metadata.

Prefer:

- cargo add
- cargo remove
- cargo update
- cargo check
- cargo test
- cargo clippy
- cargo fmt

Do not manually pin dependency versions without a reason.

Keep Cargo.lock committed because this is an application rather than a library.

When adding a dependency, explain briefly why it is needed if the reason is not
obvious.

## Quality Gate

Before considering a coding task complete, run as appropriate:

```text
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

A task is not complete merely because the code compiles. Choose focused
validation appropriate to the change, including deterministic simulation tests
and UI/viewer checks where relevant.

## Architecture

The Architect owns architectural decisions.

Other agents may propose architectural changes but should document the proposal
and ask the Architect to approve significant changes.

Significant changes include:

- selecting or changing the game engine, renderer, or UI framework
- changing the ECS or simulation-state architecture
- changing major dependencies
- changing the module boundaries
- introducing a new persistence mechanism
- introducing networking
- changing simulation timing, spatial representation, or communication model
- changing the rendering architecture

## Documentation

Before making a significant architectural change, review the relevant source
modules and `docs/DECISIONS.md`. Document accepted decisions there. Add broader
architecture documents only when they answer a concrete project need.

Do not silently contradict existing architectural decisions.

Record every accepted architectural, simulation-rule, or user-facing behavior
decision in `docs/DECISIONS.md` in the same change that implements it, including
numerical defaults and control bindings. If a decision changes, add a new entry
that explicitly supersedes the previous one; do not rely on chat history or
silently edit the old decision. Keep `docs/PROJECT_STATE.md` and the README
useful for resuming from a fresh clone.

## Testing

Simulation rules should be deterministic and testable wherever practical.

Prefer testing:

- simulation state transitions
- cell behavior and signal delivery
- feed uptake, reproduction, and withering rules
- spatial queries and field boundaries
- population/resource calculations
- seeded reproducibility and save/load behavior when introduced
- serialization/deserialization

Avoid tests that depend unnecessarily on rendering, wall-clock timing,
uncontrolled random values, or the Windows UI.

When randomness is required for simulation logic, make the random source
injectable or seedable so tests can be deterministic.

## Simulation and Visualization

Model the two intended communication channels as distinct mechanisms:

- a fast, local channel for nearby-cell communication
- a slower, wider-area channel for signals that spread through the field

Keep the first implementation abstract; biological fidelity is not a
requirement. Document unresolved numerical rules rather than inventing them
silently.

The viewer is a first-class workstream. Keep visualization concerns separate
from simulation mechanics, and coordinate UI work with the simulation state
contract. Simulation progression must not depend on rendering frame rate unless
an approved design explicitly requires it.

## Agent Collaboration

Agents must respect the ownership boundaries defined by the Architect.

Do not rewrite unrelated code.

Do not undo another agent's changes simply to impose a preferred coding style.

If a task exposes an architectural problem, document it rather than silently
creating a second architecture.

When uncertain, inspect existing code and documentation before making
assumptions.
