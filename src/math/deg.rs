use super::Rad;

pub struct Deg(pub f32);

impl Into<Rad> for Deg {
    fn into(self) -> Rad {
        Rad(cgmath::Rad::from(cgmath::Deg(self.0)).0)
    }
}
