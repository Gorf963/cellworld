# Cellworld Decision Log

This is the canonical, version-controlled record of accepted architectural
and simulation behavior decisions. Add an entry when a decision is made, not
later from memory. Record changes to existing decisions as new entries that
explicitly supersede the earlier one; do not silently rewrite history.

## Decision: Use Bevy for the initial 3D viewer

### Context

The first simulation prototype ran entirely in the console, while an
interactive 3D viewer is a core project goal. Bevy is already a project
dependency.

### Decision

Use Bevy to open the desktop window and render the initial cell-and-feed view.
Keep the simulation rules in the `cellworld` library; the viewer owns
presentation, input, and fixed-step scheduling.

### Alternatives

- Keep the console-only executable until a separate renderer is selected.
- Implement a custom renderer or use a lower-level graphics library.

### Consequences

- `cargo run` opens the interactive viewer.
- Rendering remains separate from the simulation model.
- The application inherits Bevy's runtime and platform requirements.

### Status

Accepted

## Decision: Keep simulation responsibilities in focused modules

### Context

The prototype is growing beyond a single-file simulation and needs clear
ownership boundaries for continued development.

### Decision

Keep the Rust library independent from viewer concerns and separate simulation
responsibilities into these modules:

- `field.rs`: integer 3D coordinates.
- `cells.rs`: shared cell state and the `CellType` abstraction.
- `feed.rs`: sparse feed storage.
- `signals.rs`: fast and slow signal accumulation.
- `world.rs`: simulation-step orchestration and interaction rules.
- `viewer.rs`: Bevy application, rendering, UI, input, and camera controls.

### Alternatives

- Keep all simulation and viewer code in one or two large source files.
- Split into many smaller modules before the responsibilities need it.

### Consequences

- Simulation rules can be tested without opening a window.
- Cross-cutting behavior belongs in the world orchestration, while type-
  specific behavior remains on `CellType`.
- New modules should follow these boundaries unless an updated decision says
  otherwise.

### Status

Accepted

## Decision: Use trait-based cell types

### Context

The simulation needs to support multiple cell types without coupling shared
cell state and world orchestration to each concrete kind.

### Decision

Keep common identity, position, energy, signal, and generation data in `Cell`.
Represent type-specific simulation rules with the `CellType` trait, initially
implemented by `BasicCellType`. Keep field coordinates, feed storage, signal
accumulation, and world-step orchestration in separate modules.

### Alternatives

- Put every kind in one enum, requiring central match statements when kinds are
  added.
- Duplicate common state and simulation logic across concrete cell structs.

### Consequences

- New cell kinds can provide movement range, energy use, feed uptake, and
  reproduction thresholds independently.
- World logic continues to operate on a heterogeneous population.
- Each cell type must provide a clone implementation for inherited offspring.

### Status

Accepted

## Decision: Use a deterministic fixed-step simulation baseline

### Context

Simulation progression should be independent of the viewer's render frame
rate, and the initial prototype needs concrete rules that can be tested.

### Decision

The viewer schedules one simulation step every 0.5 seconds. The current
`BasicCellType` baseline uses:

- Initial energy: 1.0.
- Energy cost per step: 0.45.
- Maximum feed uptake per step: 1.2.
- Reproduction threshold: energy greater than 2.5.
- Movement search radius: one coordinate unit on each axis.

Cells move to the available feed tile in their search cube with the greatest
amount; ties prefer the shortest Manhattan distance and then lexicographic
coordinate order. If no tile in range has positive feed, a cell stays in
place.

Feed generated per tick is divided evenly among cells at their pre-movement
positions. Cells sharing a destination tile split its available feed evenly.
Reproduction finds the first unoccupied neighboring coordinate in nested
coordinate iteration order; parent and child split the parent's energy equally,
and the child inherits the parent's concrete cell type. A cell with energy at
or below zero is removed.

These values and rules are the initial implemented baseline, not claims of
biological fidelity. Any change to them must be documented and regression
tested.

### Alternatives

- Advance simulation once per rendered frame.
- Leave all initial behavior numerical choices implicit in implementation.

### Consequences

- Simulation tests are deterministic for a given world state.
- The initial numbers are easy to find and review here before tuning.
- Per-type rule overrides remain available through `CellType`.

### Status

Accepted

## Decision: Define separate fast and slow signal channels

### Context

Cells need distinct local and wider-area communication channels while keeping
the first model simple and deterministic.

### Decision

For each cell, the fast channel adds 2.0 at the cell's coordinate and 1.0 at
every other coordinate in the 3x3x3 cube centered on it. A cell's stored fast
signal is the accumulated value at its coordinate divided by 27.

The slow channel contributes to the surrounding 5x5x5 cube except the source
coordinate. Each contribution is `1 / (Manhattan distance + 1)`. A cell's
stored slow signal is the accumulated value at its coordinate divided by 10.
Signal maps are recalculated from the current cells on each step.

### Alternatives

- Merge communication into one signal map.
- Introduce continuous propagation or diffusion before the baseline needs it.

### Consequences

- Fast local and slow wider-area signals remain distinct and separately
  testable.
- The current attenuation, neighborhood, and normalization values are
  documented as prototype rules and must be revisited explicitly if changed.

### Status

Accepted

## Decision: Stop the viewer simulation when the population is extinct

### Context

With no cells remaining, automatically generating feed or advancing ticks has
no useful effect and obscures the final simulation state.

### Decision

The viewer does not run fixed simulation steps when `cell_count()` is zero.
The visible state reports `Extinct`; users close the window to end the run.

### Alternatives

- Continue empty ticks and feed generation indefinitely.
- Automatically close the viewer on extinction.

### Consequences

- The extinction state and final tick remain visible for inspection.
- Restarting currently requires launching the application again; an in-viewer
  restart flow is not yet implemented.

### Status

Accepted

## Decision: Close the viewer automatically when the population becomes extinct

### Context

The previous extinction behavior left the viewer open after the final tick,
which made a zero-cell result look like a stuck application instead of a clean
end state.

### Decision

Supersedes the earlier extinction decision: when `cell_count()` reaches zero,
`AppExit::Success` is sent and the Bevy application exits immediately after the
final UI update. The status text still reports `Extinct` before shutdown.

### Alternatives

- Leave the application open until the user manually closes it.
- Exit before redrawing the final extinction text.

### Consequences

- The final extinction frame is visible and then the window closes cleanly.
- The process exits with a successful status code after the last cell dies.
- A restart still requires launching the app again.

### Status

Accepted

## Decision: Expose feed generation and camera controls in the viewer

### Context

Users need to adjust the simulation's feed pressure and inspect a 3D field
without editing code.

### Decision

The default feed generation is 1.5 units per tick and may be supplied as the
first command-line argument. The top-left HUD provides minus and plus controls
that change the rate by 0.1 units per click; the rate cannot go below zero.
Space pauses and resumes simulation. Left mouse drag orbits the camera, right
mouse drag pans, and the scroll wheel zooms. Escape closes the window.

### Alternatives

- Require source edits to change feed generation.
- Use keyboard-only camera controls.

### Consequences

- These user-facing controls and defaults are part of the project's behavior.
- Changes to control bindings, step size, or initial rate should be recorded
  and reflected in the README.

### Status

Accepted
