use std::collections::HashMap;

use crate::{Cell, CellType, Coord3, Feed, signals};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct World {
    next_id: usize,
    pub cells: Vec<Cell>,
    pub feed: Feed,
    pub local_signals: HashMap<Coord3, f32>,
    pub slow_signals: HashMap<Coord3, f32>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cell(&mut self, position: Coord3) -> usize {
        self.add_cell_with_type(position, crate::BasicCellType)
    }

    pub fn add_cell_with_type(
        &mut self,
        position: Coord3,
        cell_type: impl CellType + 'static,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.cells.push(Cell::with_type(id, position, 1, cell_type));
        id
    }

    pub fn add_feed(&mut self, position: Coord3, amount: f32) {
        self.feed.add(position, amount);
    }

    fn choose_move_target(&self, cell: &Cell) -> Coord3 {
        let range = cell.cell_type().movement_radius();
        let mut best_position = cell.position;
        let mut best_amount = 0.0;
        let mut best_distance = usize::MAX;
        let mut best_key = (cell.position.x, cell.position.y, cell.position.z);
        let mut found_feed = false;

        for dx in -range..=range {
            for dy in -range..=range {
                for dz in -range..=range {
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

        (self.local_signals, self.slow_signals) = signals::accumulate(&self.cells);

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
            let uptake = share.min(cell.cell_type().max_feed_uptake());
            cell.energy += uptake - cell.cell_type().energy_cost();
            self.feed.set(cell.position, available - uptake);

            if cell.energy > cell.cell_type().reproduction_threshold()
                && let Some(position) = self.find_child_position(cell.position, &next_generation)
            {
                let mut child = Cell::with_boxed_type(
                    self.next_id,
                    position,
                    cell.generation + 1,
                    cell.cell_type().clone_type(),
                );
                child.energy = cell.energy * 0.5;
                cell.energy *= 0.5;
                self.next_id += 1;
                next_generation.push(child);
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

    fn find_child_position(&self, position: Coord3, next_generation: &[Cell]) -> Option<Coord3> {
        for dx in -1isize..=1isize {
            for dy in -1isize..=1isize {
                for dz in -1isize..=1isize {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let candidate = position.offset(dx, dy, dz);
                    let occupied = self.cells.iter().any(|other| other.position == candidate)
                        || next_generation
                            .iter()
                            .any(|other| other.position == candidate);
                    if !occupied {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn total_energy(&self) -> f32 {
        self.cells.iter().map(|cell| cell.energy).sum()
    }
}
