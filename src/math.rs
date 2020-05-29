pub use cgmath::perspective;

pub type Matrix4 = cgmath::Matrix4<f32>;
pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Rad = cgmath::Rad<f32>;
pub type Euler = cgmath::Euler<Rad>;

pub mod prelude {
    pub use cgmath::{InnerSpace, SquareMatrix, Zero};
}

pub fn rad(v: f32) -> Rad {
    cgmath::Rad(v)
}
