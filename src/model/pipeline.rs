use super::{handle::ModelRef, Material, Vertex};
use cgmath::{Matrix4, Rad, Zero};
use std::{mem, sync::Arc};
use vulkano::{
    buffer::CpuBufferPool,
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    descriptor::descriptor_set::{PersistentDescriptorSet, StdDescriptorPool},
    device::{Device, Queue},
    format::R8G8B8A8Srgb,
    framebuffer::{RenderPassAbstract, Subpass},
    image::{Dimensions, ImmutableImage},
    pipeline::{GraphicsPipeline, GraphicsPipelineAbstract},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    sync::{now, GpuFuture},
};

pub struct Pipeline {
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    device: Arc<Device>,
    empty_texture: Arc<ImmutableImage<R8G8B8A8Srgb>>,
    sampler: Arc<Sampler>,
    next_frame_futures: Vec<Box<dyn GpuFuture>>,
}

impl Pipeline {
    pub fn create(
        device: Arc<Device>,
        queue: Arc<Queue>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Self {
        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .cull_mode_back()
                .blend_alpha_blending()
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );
        let uniform_buffer = CpuBufferPool::<vs::ty::Data>::uniform_buffer(device.clone());
        let (empty_texture, fut) = generate_empty_texture(queue, [255, 0, 0, 255]);

        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();

        Self {
            pipeline,
            uniform_buffer,
            device,
            empty_texture,
            sampler,
            next_frame_futures: vec![fut],
        }
    }
    pub fn render<'a>(
        &mut self,
        future: &mut Box<dyn GpuFuture>,
        models: impl Iterator<Item = &'a ModelRef>,
        command_buffer_builder: &mut AutoCommandBufferBuilder,
        dimensions: [f32; 2],
        camera: Matrix4<f32>,
        directional_lights: (i32, [vs::ty::DirectionalLight; 100]),
        dynamic_state: &DynamicState,
        descriptor_pool: &mut Arc<StdDescriptorPool>,
    ) {
        for fut in self.next_frame_futures.drain(..) {
            let tmp = std::mem::replace(future, now(self.device.clone()).boxed());
            *future = tmp.join(fut).boxed();
        }
        let proj = cgmath::perspective(
            Rad(std::f32::consts::FRAC_PI_2),
            dimensions[0] / dimensions[1],
            0.01,
            100.0,
        );

        let mut data = default_uniform(camera, proj, directional_lights);

        for model in models {
            let model_data = model.data.read();
            let model = &model.model;
            let base_matrix = model_data.matrix();

            if !model.texture_future.read().is_empty() {
                let texture_futures = mem::replace(&mut *model.texture_future.write(), Vec::new());
                for fut in texture_futures {
                    let tmp = std::mem::replace(future, now(self.device.clone()).boxed());
                    *future = tmp.join(fut).boxed();
                }
            }
            let layout = self.pipeline.descriptor_set_layout(0).unwrap();

            for (group, group_data) in model.groups.iter().zip(model_data.groups.iter()) {
                let texture = group
                    .texture
                    .as_ref()
                    .unwrap_or(&self.empty_texture)
                    .clone();

                data.world = (base_matrix * group_data.matrix).into();
                update_uniform_material(&mut data, group.material.as_ref());

                let uniform_buffer_subbuffer = self.uniform_buffer.next(data).unwrap();

                let set = Arc::new(
                    PersistentDescriptorSet::start(layout.clone())
                        .add_buffer(uniform_buffer_subbuffer)
                        .unwrap()
                        .add_sampled_image(texture, self.sampler.clone())
                        .unwrap()
                        .build_with_pool(descriptor_pool)
                        .unwrap(),
                );

                let vertex_buffer = group
                    .vertex_buffer
                    .as_ref()
                    .or_else(|| model.vertex_buffer.as_ref())
                    .expect("Model has no valid vertex buffer");

                if let Some(index) = group.index.as_ref() {
                    command_buffer_builder
                        .draw_indexed(
                            self.pipeline.clone(),
                            dynamic_state,
                            vec![vertex_buffer.clone()],
                            index.clone(),
                            set.clone(),
                            (),
                        )
                        .unwrap();
                } else {
                    command_buffer_builder
                        .draw(
                            self.pipeline.clone(),
                            dynamic_state,
                            vec![vertex_buffer.clone()],
                            set,
                            (),
                        )
                        .unwrap();
                }
            }
        }
    }
}

fn default_uniform(
    camera: Matrix4<f32>,
    proj: Matrix4<f32>,
    directional_lights: (i32, [vs::ty::DirectionalLight; 100]),
) -> vs::ty::Data {
    let camera_pos = -camera.z.truncate();

    vs::ty::Data {
        world: Matrix4::zero().into(),
        view: camera.into(),
        proj: proj.into(),
        lights: directional_lights.1,
        lightCount: directional_lights.0,

        camera_x: camera_pos.x,
        camera_y: camera_pos.y,
        camera_z: camera_pos.z,
        material_ambient_r: 0.0,
        material_ambient_g: 0.0,
        material_ambient_b: 0.0,
        material_diffuse_r: 0.0,
        material_diffuse_g: 0.0,
        material_diffuse_b: 0.0,
        material_specular_r: 0.0,
        material_specular_g: 0.0,
        material_specular_b: 0.0,
        material_shininess: 0.0,
    }
}
pub(crate) fn update_uniform_material(data: &mut vs::ty::Data, material: Option<&Material>) {
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

fn generate_empty_texture(
    queue: Arc<Queue>,
    color: [u8; 4],
) -> (Arc<ImmutableImage<R8G8B8A8Srgb>>, Box<dyn GpuFuture>) {
    let (img, fut) = ImmutableImage::from_iter(
        color.iter().cloned(),
        Dimensions::Dim2d {
            width: 1,
            height: 1,
        },
        R8G8B8A8Srgb,
        queue,
    )
    .unwrap();
    (img, fut.boxed())
}
