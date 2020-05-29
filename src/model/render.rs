use super::data::ModelDataGroup;
use crate::{
    math::Matrix4,
    render::{Material, RenderPipeline},
};
use std::{mem, sync::Arc};
use vulkano::{
    command_buffer::AutoCommandBufferBuilder, descriptor::descriptor_set::PersistentDescriptorSet,
    sync::GpuFuture,
};

impl super::Model {
    pub fn render(
        &self,
        mut future: Box<dyn GpuFuture>,
        groups: &[ModelDataGroup],
        base_matrix: Matrix4,
        data: &mut vs::ty::Data,
        mut command_buffer_builder: AutoCommandBufferBuilder,

        pipeline: &mut RenderPipeline,
    ) -> (AutoCommandBufferBuilder, Box<dyn GpuFuture>) {
        if !self.texture_future.read().is_empty() {
            let texture_futures = mem::replace(&mut *self.texture_future.write(), Vec::new());
            for fut in texture_futures {
                future = Box::new(future.join(fut)) as _;
            }
        }
        let layout = pipeline.pipeline.descriptor_set_layout(0).unwrap();

        for (group, group_data) in self.groups.iter().zip(groups.iter()) {
            let texture = group
                .texture
                .as_ref()
                .unwrap_or(&pipeline.empty_texture)
                .clone();

            data.world = (base_matrix * group_data.matrix).into();
            update_uniform_material(data, group.material.as_ref());

            let uniform_buffer_subbuffer = pipeline.uniform_buffer.next(*data).unwrap();

            let set = Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .add_sampled_image(texture, pipeline.sampler.clone())
                    .unwrap()
                    .build_with_pool(&mut pipeline.descriptor_pool)
                    .unwrap(),
            );

            let vertex_buffer = group
                .vertex_buffer
                .as_ref()
                .or_else(|| self.vertex_buffer.as_ref())
                .expect("Model has no valid vertex buffer");

            command_buffer_builder = if let Some(index) = group.index.as_ref() {
                command_buffer_builder
                    .draw_indexed(
                        pipeline.pipeline.clone(),
                        &pipeline.dynamic_state,
                        vec![vertex_buffer.clone()],
                        index.clone(),
                        set.clone(),
                        (),
                    )
                    .unwrap()
            } else {
                command_buffer_builder
                    .draw(
                        pipeline.pipeline.clone(),
                        &pipeline.dynamic_state,
                        vec![vertex_buffer.clone()],
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

vec3 max_member(vec3 lhs, vec3 rhs) {
    return vec3(
        max(lhs.x, rhs.x),
        max(lhs.y, rhs.y),
        max(lhs.z, rhs.z)
    );
}

vec4 min_member(vec4 lhs, vec4 rhs) {
    return vec4(
        min(lhs.x, rhs.x),
        min(lhs.y, rhs.y),
        min(lhs.z, rhs.z),
        min(lhs.w, rhs.w)
    );
}

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
    return tex_color * min_member(vec4(ambient + diffuse + specular, 1.0), vec4(1.0, 1.0, 1.0, 1.0));
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
