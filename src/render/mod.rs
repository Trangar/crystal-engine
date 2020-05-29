mod lights;
mod pipeline;
mod window;

pub use self::{lights::*, pipeline::*, window::*};

// TODO: Make it so that developers can create their own models/vertices?
#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position_in: [f32; 3],
    pub normal_in: [f32; 3],
    pub tex_coord_in: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position_in, normal_in, tex_coord_in);

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambient: [1.0, 1.0, 1.0],
            diffuse: [1.0, 1.0, 1.0],
            specular: [1.0, 1.0, 1.0],
            shininess: 1.0,
        }
    }
}
