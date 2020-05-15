mod builder;
mod data;
mod handle;
mod loader;

pub use self::{
    builder::ModelBuilder,
    data::ModelData,
    handle::{ModelHandle, ModelHandleMessage},
    loader::SourceOrShape,
};

use super::{Material, Vertex};
use loader::ParsedModelPart;
use parking_lot::RwLock;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    format::R8G8B8A8Srgb,
    image::ImmutableImage,
    sync::GpuFuture,
};

// TODO: Make it so that developers can create their own models/vertices?
pub struct Model {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub groups: Vec<ModelGroup>,
    pub texture_future: RwLock<Vec<Box<dyn GpuFuture>>>,
}

pub struct ModelGroup {
    pub material: Option<Material>,
    pub texture: Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>,
    pub index: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
}

impl ModelGroup {
    pub fn from_part(
        device: Arc<Device>,
        texture: &Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>,
        part: ParsedModelPart,
    ) -> (Self, Option<Box<dyn GpuFuture>>) {
        let index = Some(
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                part.index.iter().copied(),
            )
            .unwrap(),
        );

        (
            Self {
                material: None,
                texture: texture.clone(),
                index,
            },
            None,
        )
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "#version 450

layout(location = 0) in vec3 position_in;
layout(location = 1) in vec3 normal_in;
layout(location = 2) in vec2 tex_coord_in;

layout(location = 0) out vec2 fragment_tex_coord;
layout(location = 1) out vec3 fragment_normal;

struct DirectionalLight {
    vec3 direction;
    vec4 color;
};

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    DirectionalLight[100] lights;
    int lightCount;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(position_in, 1.0);
    fragment_tex_coord = tex_coord_in;

    fragment_normal = transpose(inverse(mat3(worldview))) * normal_in;
}
"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 450

layout(location = 0) in vec2 fragment_tex_coord;
layout(location = 1) in vec3 fragment_normal;

layout(location = 0) out vec4 f_color;

struct DirectionalLight {
    vec3 direction;
    vec4 color;
};

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    DirectionalLight[100] lights;
    int lightCount;
} uniforms;

void main() {
    if(fragment_tex_coord.x < 0.0 && fragment_tex_coord.y < 0.0) {
        f_color = vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        f_color = texture(tex, fragment_tex_coord);
    }
    
    vec4 light_color = vec4(0.0, 0.0, 0.0, 1.0); 
    for(int i = 0; i < uniforms.lightCount; i++) {
        DirectionalLight light = uniforms.lights[i];
        vec3 direction = vec3(light.direction.x, -light.direction.y, light.direction.z);
        float brightness = dot(normalize(fragment_normal), normalize(direction));
        vec4 color = light.color * brightness;
        light_color = vec4(
            max(light_color.x, color.x),
            max(light_color.y, color.y),
            max(light_color.z, color.z),
            1.0
        );
    }
    f_color *= light_color;
}
"
    }
}
