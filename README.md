# Cellworld

Cellworld is an early Rust simulation prototype with a 3D cell-and-feed
simulation viewer.

## Requirements

- Stable Rust with the Rust 2024 edition
- Cargo (included with the Rust toolchain)

## Run

From the repository directory:

```powershell
cargo run
```

The application opens a window that displays cells as green spheres and feed
as amber cubes. The simulation advances every half-second; press Space to
pause or resume, and Esc to close the window. The default feed generation is
`1.5` units per tick.

Use the **− / +** controls in the top-left panel to decrease or increase
generated feed by `0.1` units per tick. Feed generation cannot be reduced below
zero.
The simulation stops advancing once the last cell dies.

Use the mouse to navigate the field: left-drag to rotate, right-drag to pan,
and scroll to zoom.

Simulation code is organized by responsibility:

- `src/field.rs` defines 3D field coordinates.
- `src/cells.rs` defines shared cell state and the `CellType` abstraction.
- `src/feed.rs` owns feed storage.
- `src/signals.rs` calculates fast and slow signals.
- `src/world.rs` coordinates simulation steps.
- `src/viewer.rs` contains the Bevy viewer and camera controls.

New cell kinds implement `CellType` and can override movement range, energy
use, feed uptake, and reproduction threshold. Offspring inherit the parent
kind. For example:

```rust
#[derive(Clone, Debug)]
struct EfficientCell;

impl cellworld::CellType for EfficientCell {
    fn name(&self) -> &'static str {
        "Efficient"
    }

    fn energy_cost(&self) -> f32 {
        0.1
    }

    fn clone_type(&self) -> Box<dyn cellworld::CellType> {
        Box::new(self.clone())
    }
}
```

Pass a number to change the amount of feed generated each tick:

```powershell
cargo run -- 0.5
```

For example, `cargo run -- 0` disables ongoing feed generation. The simulation
starts with a small amount of feed already in the world.

## Project continuity

See [docs/PROJECT_STATE.md](docs/PROJECT_STATE.md) for the project map and
fresh-clone handoff. [docs/DECISIONS.md](docs/DECISIONS.md) is the versioned
record of accepted architecture and simulation behavior; update it whenever a
decision changes or a new one is made.

## Validate

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```
