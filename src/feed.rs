use std::collections::HashMap;

use crate::Coord3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Feed {
    amounts: HashMap<Coord3, f32>,
}

impl Feed {
    pub fn add(&mut self, position: Coord3, amount: f32) {
        let clamped = amount.clamp(0.0, 100.0);
        *self.amounts.entry(position).or_insert(0.0) += clamped;
    }

    pub fn get(&self, position: &Coord3) -> Option<&f32> {
        self.amounts.get(position)
    }

    pub fn set(&mut self, position: Coord3, amount: f32) {
        self.amounts.insert(position, amount);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Coord3, &f32)> {
        self.amounts.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &f32> {
        self.amounts.values()
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&Coord3, &mut f32) -> bool) {
        self.amounts
            .retain(|position, amount| keep(position, amount));
    }
}
