use std::any::Any;
use std::fmt::Debug;

use crate::Coord3;

pub trait CellType: Any + Debug + Send + Sync {
    fn name(&self) -> &'static str;

    fn initial_energy(&self) -> f32 {
        1.0
    }

    fn energy_cost(&self) -> f32 {
        0.45
    }

    fn max_feed_uptake(&self) -> f32 {
        1.2
    }

    fn reproduction_threshold(&self) -> f32 {
        2.5
    }

    fn movement_radius(&self) -> isize {
        1
    }

    fn clone_type(&self) -> Box<dyn CellType>;
}

impl Clone for Box<dyn CellType> {
    fn clone(&self) -> Self {
        self.clone_type()
    }
}

impl PartialEq for dyn CellType {
    fn eq(&self, other: &Self) -> bool {
        Any::type_id(self) == Any::type_id(other)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BasicCellType;

impl CellType for BasicCellType {
    fn name(&self) -> &'static str {
        "Basic"
    }

    fn clone_type(&self) -> Box<dyn CellType> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub id: usize,
    pub position: Coord3,
    pub energy: f32,
    pub fast_signal: f32,
    pub slow_signal: f32,
    pub generation: u32,
    cell_type: Box<dyn CellType>,
}

impl Cell {
    pub fn new(id: usize, position: Coord3, generation: u32) -> Self {
        Self::with_type(id, position, generation, BasicCellType)
    }

    pub fn with_type(
        id: usize,
        position: Coord3,
        generation: u32,
        cell_type: impl CellType + 'static,
    ) -> Self {
        Self::with_boxed_type(id, position, generation, Box::new(cell_type))
    }

    pub(crate) fn with_boxed_type(
        id: usize,
        position: Coord3,
        generation: u32,
        cell_type: Box<dyn CellType>,
    ) -> Self {
        let energy = cell_type.initial_energy();
        Self {
            id,
            position,
            energy,
            fast_signal: 0.0,
            slow_signal: 0.0,
            generation,
            cell_type,
        }
    }

    pub fn cell_type(&self) -> &dyn CellType {
        self.cell_type.as_ref()
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.position == other.position
            && self.energy == other.energy
            && self.fast_signal == other.fast_signal
            && self.slow_signal == other.slow_signal
            && self.generation == other.generation
            && self.cell_type.as_ref() == other.cell_type.as_ref()
    }
}
