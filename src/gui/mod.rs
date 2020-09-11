mod builder;
mod element;
mod pipeline;

pub use self::{
    builder::{GuiElementBuilder, GuiElementCanvasBuilder, GuiElementTextureBuilder},
    element::{ElementId, GuiElement, GuiElementData, GuiElementRef},
    pipeline::Pipeline,
};

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub offset: [f32; 2],
    pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, offset, tex_coord);

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 450
layout(location = 0) in vec2 offset;
layout(location = 2) in vec2 tex_coord;

layout(location = 0) out vec2 fragment_tex_coord;

layout(set = 0, binding = 0) uniform Data {
    vec2 screen_size;
    vec2 position;
    vec2 size;
} uniforms;

void main() {
    vec2 half_screen_size = uniforms.screen_size / 2;

    gl_Position = vec4(
        (uniforms.position / half_screen_size - vec2(1.0, 1.0)) +
        (offset * uniforms.size / half_screen_size),
        0.0, 1.0);
    fragment_tex_coord = tex_coord;
}
"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 450

layout(location = 0) in vec2 fragment_tex_coord;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform Data {
    vec2 screen_size;
    vec2 position;
    vec2 size;
} uniforms;
layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
    f_color = texture(tex, fragment_tex_coord);
}
"
    }
}
