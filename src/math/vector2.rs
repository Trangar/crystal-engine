#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn perp_dot(self, other: Self) -> f32 {
        cgmath::Vector2::new(self.x, self.y).perp_dot(cgmath::Vector2::new(other.x, other.y))
    }
}

impl Into<[f32; 2]> for Vector2 {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
