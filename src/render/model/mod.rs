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
    float direction_x;
    float direction_y;
    float direction_z;
    float color_ambient_r;
    float color_ambient_g;
    float color_ambient_b;
    float color_diffuse_r;
    float color_diffuse_g;
    float color_diffuse_b;
    float color_specular_r;
    float color_specular_g;
    float color_specular_b;
};

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    DirectionalLight[100] lights;
    int lightCount;

    float camera_x;
    float camera_y;
    float camera_z;

    float material_ambient_r;
    float material_ambient_g;
    float material_ambient_b;
    float material_diffuse_r;
    float material_diffuse_g;
    float material_diffuse_b;
    float material_specular_r;
    float material_specular_g;
    float material_specular_b;
    float material_shininess;
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
    float direction_x;
    float direction_y;
    float direction_z;
    float color_ambient_r;
    float color_ambient_g;
    float color_ambient_b;
    float color_diffuse_r;
    float color_diffuse_g;
    float color_diffuse_b;
    float color_specular_r;
    float color_specular_g;
    float color_specular_b;
};

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    DirectionalLight[100] lights;
    int lightCount;

    float camera_x;
    float camera_y;
    float camera_z;

    float material_ambient_r;
    float material_ambient_g;
    float material_ambient_b;
    float material_diffuse_r;
    float material_diffuse_g;
    float material_diffuse_b;
    float material_specular_r;
    float material_specular_g;
    float material_specular_b;
    float material_shininess;
} uniforms;

vec4 CalcDirLight(DirectionalLight light, vec4 tex_color, vec3 normal, vec3 viewDir)
{
    vec3 direction = vec3(light.direction_x, light.direction_y, light.direction_z);
    vec3 ambient = vec3(light.color_ambient_r, light.color_ambient_g, light.color_ambient_b);
    vec3 diffuse = vec3(light.color_diffuse_r, light.color_diffuse_g, light.color_diffuse_b);
    vec3 specular = vec3(light.color_specular_r, light.color_specular_g, light.color_specular_b);

    vec3 material_ambient = vec3(uniforms.material_ambient_r, uniforms.material_ambient_g, uniforms.material_ambient_b);
    vec3 material_diffuse = vec3(uniforms.material_diffuse_r, uniforms.material_diffuse_g, uniforms.material_diffuse_b);
    vec3 material_specular = vec3(uniforms.material_specular_r, uniforms.material_specular_g, uniforms.material_specular_b);

    vec3 lightDir = normalize(-direction);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), uniforms.material_shininess);
    // combine results
    ambient  = ambient  * material_ambient;
    diffuse  = diffuse  * diff * material_diffuse;
    specular = specular * spec * material_specular;
    return tex_color * vec4(ambient + diffuse + specular, 1.0);
} 

vec3 max_member(vec3 lhs, vec3 rhs) {
    return vec3(
        max(lhs.x, rhs.x),
        max(lhs.y, rhs.y),
        max(lhs.z, rhs.z)
    );
}

void main() {
    if(fragment_tex_coord.x < 0.0 && fragment_tex_coord.y < 0.0) {
        f_color = vec4(uniforms.material_ambient_r, uniforms.material_ambient_g, uniforms.material_ambient_b, 1);
    } else {
        f_color = texture(tex, fragment_tex_coord);
    }

    vec3 camera_pos = vec3(uniforms.camera_x, uniforms.camera_y, uniforms.camera_z);
    
    for(int i = 0; i < uniforms.lightCount; i++) {
        f_color = CalcDirLight(
            uniforms.lights[i],
            f_color,
            fragment_normal,
            camera_pos
        );
    }
}
"
    }
}
