mod lights;
mod model;
mod pipeline;
mod window;

pub use self::{lights::*, model::*, pipeline::*, window::*};

// TODO: Make it so that developers can create their own models/vertices?
#[derive(Default, Copy, Clone)]
pub struct Vertex {
    position_in: [f32; 3],
    normal_in: [f32; 3],
    tex_coord_in: [f32; 2],
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
