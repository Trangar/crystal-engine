use super::Rad;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Euler {
    pub x: Rad,
    pub y: Rad,
    pub z: Rad,
}

impl Euler {
    pub fn new(x: Rad, y: Rad, z: Rad) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(Rad(0.0), Rad(0.0), Rad(0.0))
    }
}
