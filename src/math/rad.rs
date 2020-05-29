#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Rad(pub f32);

impl Rad {
    pub fn zero() -> Rad {
        Rad(0.0)
    }
    pub fn turn_div_2() -> Rad {
        use cgmath::Angle;
        Rad(cgmath::Rad::turn_div_2().0)
    }

    pub fn cot(self) -> f32 {
        use cgmath::Angle;
        cgmath::Rad::cot(cgmath::Rad(self.0))
    }
}

impl std::ops::Div for Rad {
    type Output = Rad;
    fn div(self, other: Self) -> Rad {
        Rad(self.0 / other.0)
    }
}

impl std::ops::Div<f32> for Rad {
    type Output = Rad;
    fn div(self, other: f32) -> Rad {
        Rad(self.0 / other)
    }
}

impl std::ops::Add for Rad {
    type Output = Rad;
    fn add(self, other: Rad) -> Rad {
        Rad(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Rad {
    fn add_assign(&mut self, other: Rad) {
        *self = Rad(self.0 + other.0)
    }
}
