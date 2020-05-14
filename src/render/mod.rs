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
