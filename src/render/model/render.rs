use super::data::ModelDataGroup;
use crate::render::Material;
use cgmath::Matrix4;
use std::{mem, sync::Arc};
use vulkano::format::R8G8B8A8Srgb;
use vulkano::{
    buffer::CpuBufferPool,
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    descriptor::descriptor_set::{PersistentDescriptorSet, UnsafeDescriptorSetLayout},
    image::ImmutableImage,
    pipeline::GraphicsPipelineAbstract,
    sampler::Sampler,
    sync::GpuFuture,
};

impl super::Model {
    pub fn render(
        &self,
        mut future: Box<dyn GpuFuture>,
        groups: &[ModelDataGroup],
        base_matrix: Matrix4<f32>,
        empty_texture: &Arc<ImmutableImage<R8G8B8A8Srgb>>,
        uniform_buffer: &mut CpuBufferPool<vs::ty::Data>,
        data: &mut vs::ty::Data,
        layout: &Arc<UnsafeDescriptorSetLayout>,
        sampler: &Arc<Sampler>,
        mut command_buffer_builder: AutoCommandBufferBuilder,
        pipeline: &Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
        dynamic_state: &DynamicState,
    ) -> (AutoCommandBufferBuilder, Box<dyn GpuFuture>) {
        if !self.texture_future.read().is_empty() {
            let texture_futures = mem::replace(&mut *self.texture_future.write(), Vec::new());
            for fut in texture_futures {
                future = Box::new(future.join(fut)) as _;
            }
        }

        for (group, group_data) in self.groups.iter().zip(groups.iter()) {
            let texture = group.texture.as_ref().unwrap_or(empty_texture).clone();

            data.world = (base_matrix * group_data.matrix).into();
            update_uniform_material(data, group.material.as_ref());

            let uniform_buffer_subbuffer = uniform_buffer.next(*data).unwrap();

            // TODO: We should probably cache the set in a pool
            // From the documentation: "Creating a persistent descriptor set allocates from a pool,
            // and can't be modified once created. You are therefore encouraged to create them at
            // initialization and not the during performance-critical paths."
            // 1. Create an https://docs.rs/vulkano/0.18.0/vulkano/descriptor/descriptor_set/struct.StdDescriptorPool.html
            // 2. Import this trait: https://docs.rs/vulkano/0.18.0/vulkano/descriptor/descriptor_set/trait.DescriptorPool.html
            // 3. Allocate an https://docs.rs/vulkano/0.18.0/vulkano/descriptor/descriptor_set/struct.StdDescriptorPoolAlloc.html and store it in modelhandle.
            // 4. replace .build() with .build_with_pool:
            //    https://docs.rs/vulkano/0.18.0/vulkano/descriptor/descriptor_set/struct.PersistentDescriptorSetBuilder.html#method.build_with_pool
            let set = Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .add_sampled_image(texture, sampler.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            command_buffer_builder = if let Some(index) = group.index.as_ref() {
                command_buffer_builder
                    .draw_indexed(
                        pipeline.clone(),
                        dynamic_state,
                        vec![self.vertex_buffer.clone()],
                        index.clone(),
                        set.clone(),
                        (),
                    )
                    .unwrap()
            } else {
                command_buffer_builder
                    .draw(
                        pipeline.clone(),
                        dynamic_state,
                        vec![self.vertex_buffer.clone()],
                        set,
                        (),
                    )
                    .unwrap()
            };
        }

        (command_buffer_builder, future)
    }
}

fn update_uniform_material(data: &mut vs::ty::Data, material: Option<&Material>) {
    let material = material.cloned().unwrap_or_default();
    data.material_ambient_r = material.ambient[0];
    data.material_ambient_g = material.ambient[1];
    data.material_ambient_b = material.ambient[2];
    data.material_specular_r = material.specular[0];
    data.material_specular_g = material.specular[1];
    data.material_specular_b = material.specular[2];
    data.material_diffuse_r = material.diffuse[0];
    data.material_diffuse_g = material.diffuse[1];
    data.material_diffuse_b = material.diffuse[2];
    data.material_shininess = material.shininess;
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
