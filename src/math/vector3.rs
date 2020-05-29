#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn cross(self, rhs: Self) -> Self {
        let res = cgmath::Vector3::new(self.x, self.y, self.z)
            .cross(cgmath::Vector3::new(rhs.x, rhs.y, rhs.z));
        Self::new(res.x, res.y, res.z)
    }

    pub fn dot(self, rhs: Self) -> f32 {
        use cgmath::InnerSpace;
        cgmath::Vector3::new(self.x, self.y, self.z).dot(cgmath::Vector3::new(rhs.x, rhs.y, rhs.z))
    }

    pub fn memberwise_min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
            z: self.z.min(rhs.z),
        }
    }
    pub fn memberwise_max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
            z: self.z.max(rhs.z),
        }
    }
}

impl From<(f32, f32, f32)> for Vector3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self::new(x, y, z)
    }
}

impl std::ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Into<[f32; 3]> for Vector3 {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl std::ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Into<cgmath::Point3<f32>> for Vector3 {
    fn into(self) -> cgmath::Point3<f32> {
        cgmath::Point3::new(self.x, self.y, self.z)
    }
}
