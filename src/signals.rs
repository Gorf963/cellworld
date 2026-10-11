use std::collections::HashMap;

use crate::{Cell, Coord3};

pub(crate) fn accumulate(cells: &[Cell]) -> (HashMap<Coord3, f32>, HashMap<Coord3, f32>) {
    let mut local_signals = HashMap::new();
    let mut slow_signals = HashMap::new();

    for cell in cells {
        for dx in -1isize..=1isize {
            for dy in -1isize..=1isize {
                for dz in -1isize..=1isize {
                    let position = cell.position.offset(dx, dy, dz);
                    let contribution = if dx == 0 && dy == 0 && dz == 0 {
                        2.0
                    } else {
                        1.0
                    };
                    *local_signals.entry(position).or_insert(0.0) += contribution;
                }
            }
        }
    }

    for cell in cells {
        for dx in -2isize..=2isize {
            for dy in -2isize..=2isize {
                for dz in -2isize..=2isize {
                    let distance = dx.unsigned_abs() + dy.unsigned_abs() + dz.unsigned_abs();
                    if distance == 0 {
                        continue;
                    }

                    let position = cell.position.offset(dx, dy, dz);
                    let weight = 1.0 / (distance as f32 + 1.0);
                    *slow_signals.entry(position).or_insert(0.0) += weight;
                }
            }
        }
    }

    (local_signals, slow_signals)
}
