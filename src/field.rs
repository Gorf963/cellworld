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
