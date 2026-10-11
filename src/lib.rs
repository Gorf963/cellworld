use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Coord3 {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl Coord3 {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }

    pub fn offset(self, dx: isize, dy: isize, dz: isize) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub id: usize,
    pub position: Coord3,
    pub energy: f32,
    pub fast_signal: f32,
    pub slow_signal: f32,
    pub generation: u32,
}

impl Cell {
    pub fn new(id: usize, position: Coord3, generation: u32) -> Self {
        Self {
            id,
            position,
            energy: 1.0,
            fast_signal: 0.0,
            slow_signal: 0.0,
            generation,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct World {
    next_id: usize,
    pub cells: Vec<Cell>,
    pub feed: HashMap<Coord3, f32>,
    pub local_signals: HashMap<Coord3, f32>,
    pub slow_signals: HashMap<Coord3, f32>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cell(&mut self, position: Coord3) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.cells.push(Cell::new(id, position, 1));
        id
    }

    pub fn add_feed(&mut self, position: Coord3, amount: f32) {
        let clamped = amount.clamp(0.0, 100.0);
        *self.feed.entry(position).or_insert(0.0) += clamped;
    }

    fn choose_move_target(&self, cell: &Cell) -> Coord3 {
        let mut best_position = cell.position;
        let mut best_amount = 0.0;
        let mut best_distance = usize::MAX;
        let mut best_key = (cell.position.x, cell.position.y, cell.position.z);
        let mut found_feed = false;

        for dx in -1isize..=1isize {
            for dy in -1isize..=1isize {
                for dz in -1isize..=1isize {
                    let candidate = cell.position.offset(dx, dy, dz);
                    let amount = self.feed.get(&candidate).copied().unwrap_or(0.0);
                    if amount <= 0.0 {
                        continue;
                    }

                    let distance = dx.unsigned_abs() + dy.unsigned_abs() + dz.unsigned_abs();
                    let key = (candidate.x, candidate.y, candidate.z);

                    let should_replace = !found_feed
                        || amount > best_amount
                        || (amount == best_amount
                            && (distance < best_distance
                                || (distance == best_distance && key < best_key)));

                    if should_replace {
                        best_position = candidate;
                        best_amount = amount;
                        best_distance = distance;
                        best_key = key;
                        found_feed = true;
                    }
                }
            }
        }

        best_position
    }

    fn accumulate_signals(&mut self) {
        self.local_signals.clear();
        self.slow_signals.clear();

        for cell in &self.cells {
            for dx in -1isize..=1isize {
                for dy in -1isize..=1isize {
                    for dz in -1isize..=1isize {
                        let signal_position = cell.position.offset(dx, dy, dz);
                        let contribution = if dx == 0 && dy == 0 && dz == 0 {
                            2.0
                        } else {
                            1.0
                        };
                        *self.local_signals.entry(signal_position).or_insert(0.0) += contribution;
                    }
                }
            }
        }

        for cell in &self.cells {
            for dx in -2isize..=2isize {
                for dy in -2isize..=2isize {
                    for dz in -2isize..=2isize {
                        let distance = dx.unsigned_abs() + dy.unsigned_abs() + dz.unsigned_abs();
                        if distance == 0 {
                            continue;
                        }

                        let signal_position = cell.position.offset(dx, dy, dz);
                        let weight = 1.0 / (distance as f32 + 1.0);
                        *self.slow_signals.entry(signal_position).or_insert(0.0) += weight;
                    }
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.step_with_feed(0.0);
    }

    pub fn step_with_feed(&mut self, feed_per_tick: f32) {
        if feed_per_tick > 0.0 {
            let positions: Vec<Coord3> = self.cells.iter().map(|cell| cell.position).collect();
            let per_cell = feed_per_tick / self.cells.len().max(1) as f32;
            for position in positions {
                self.add_feed(position, per_cell);
            }
        }

        let mut moved_cells = Vec::with_capacity(self.cells.len());
        for mut cell in std::mem::take(&mut self.cells) {
            cell.position = self.choose_move_target(&cell);
            moved_cells.push(cell);
        }
        self.cells = moved_cells;

        self.accumulate_signals();

        let mut tile_counts: HashMap<Coord3, usize> = HashMap::new();
        for cell in &self.cells {
            *tile_counts.entry(cell.position).or_insert(0) += 1;
        }

        let mut next_generation: Vec<Cell> = Vec::new();
        for mut cell in std::mem::take(&mut self.cells) {
            let local = self
                .local_signals
                .get(&cell.position)
                .copied()
                .unwrap_or(0.0);
            let slow = self
                .slow_signals
                .get(&cell.position)
                .copied()
                .unwrap_or(0.0);
            cell.fast_signal = local / 27.0;
            cell.slow_signal = slow / 10.0;

            let tile_count = tile_counts.get(&cell.position).copied().unwrap_or(1);
            let available = self.feed.get(&cell.position).copied().unwrap_or(0.0);
            let share = available / tile_count as f32;
            let uptake = share.min(1.2);
            let new_energy = cell.energy + uptake - 0.45;
            cell.energy = new_energy;
            self.feed.insert(cell.position, available - uptake);

            if cell.energy > 2.5 {
                let mut child_position = None;
                for dx in -1isize..=1isize {
                    for dy in -1isize..=1isize {
                        for dz in -1isize..=1isize {
                            if dx == 0 && dy == 0 && dz == 0 {
                                continue;
                            }
                            let candidate = cell.position.offset(dx, dy, dz);
                            let occupied =
                                self.cells.iter().any(|other| other.position == candidate)
                                    || next_generation
                                        .iter()
                                        .any(|other| other.position == candidate)
                                    || candidate == cell.position;
                            if !occupied {
                                child_position = Some(candidate);
                                break;
                            }
                        }
                        if child_position.is_some() {
                            break;
                        }
                    }
                    if child_position.is_some() {
                        break;
                    }
                }

                if let Some(position) = child_position {
                    let mut child = Cell::new(self.next_id, position, cell.generation + 1);
                    child.energy = cell.energy * 0.5;
                    cell.energy *= 0.5;
                    self.next_id += 1;
                    next_generation.push(child);
                }
            }

            if cell.energy > 0.0 {
                next_generation.push(cell);
            }
        }

        self.cells = next_generation;
        self.feed.retain(|_, amount| *amount > 0.01);
        self.local_signals.retain(|_, amount| *amount > 0.01);
        self.slow_signals.retain(|_, amount| *amount > 0.01);
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn total_energy(&self) -> f32 {
        self.cells.iter().map(|cell| cell.energy).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::{Coord3, World};

    #[test]
    fn cells_consume_feed_at_their_position() {
        let mut world = World::new();
        let position = Coord3::new(0, 0, 0);
        world.add_feed(position, 2.0);
        world.add_cell(position);

        world.step();

        let cell = world.cells.first().expect("cell should survive");
        assert!(cell.energy > 0.0);
        assert!(world.feed.get(&position).copied().unwrap_or_default() < 2.0);
    }

    #[test]
    fn nearby_cells_raise_local_signal() {
        let mut world = World::new();
        let a = Coord3::new(0, 0, 0);
        let b = Coord3::new(1, 0, 0);
        world.add_cell(a);
        world.add_cell(b);

        world.step();

        let first = world.cells.iter().find(|cell| cell.position == a).unwrap();
        let second = world.cells.iter().find(|cell| cell.position == b).unwrap();

        assert!(first.fast_signal > 0.0);
        assert!(second.fast_signal > 0.0);
    }

    #[test]
    fn cells_die_without_feed() {
        let mut world = World::new();
        let position = Coord3::new(4, 4, 4);
        world.add_cell(position);

        for _ in 0..5 {
            world.step();
        }

        assert_eq!(world.cell_count(), 0);
    }

    #[test]
    fn cells_move_towards_available_feed() {
        let mut world = World::new();
        world.add_cell(Coord3::new(0, 0, 0));
        world.add_feed(Coord3::new(1, 0, 0), 3.0);

        world.step();

        let cell = world.cells.first().expect("cell should still exist");
        assert_eq!(cell.position, Coord3::new(1, 0, 0));
    }

    #[test]
    fn cells_reproduce_when_energy_is_high() {
        let mut world = World::new();
        world.add_feed(Coord3::new(0, 0, 0), 20.0);
        world.add_cell(Coord3::new(0, 0, 0));

        for _ in 0..12 {
            world.add_feed(Coord3::new(0, 0, 0), 20.0);
            world.step();
            if world.cell_count() > 1 {
                break;
            }
        }

        assert!(
            world.cell_count() >= 2,
            "fed cells should reproduce once they have enough energy"
        );
    }

    #[test]
    fn feed_can_be_generated_per_tick() {
        let mut world = World::new();
        world.add_cell(Coord3::new(0, 0, 0));

        world.step_with_feed(2.5);

        let cell = world.cells.first().expect("cell should exist");
        assert!(world.feed.get(&cell.position).copied().unwrap_or_default() > 0.0);
    }
}
