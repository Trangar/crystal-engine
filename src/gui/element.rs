use super::Vertex;
use crate::{
    model::{vs, Model, ModelDataGroup, ModelHandle},
    render::RenderPipeline,
};
use cgmath::Matrix4;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::AutoCommandBufferBuilder,
    descriptor::descriptor_set::PersistentDescriptorSet,
    device::Device,
    sync::{now, GpuFuture},
};

pub struct GuiElement(pub Arc<Model>);

impl GuiElement {
    pub fn render(
        &self,
        future: &mut Box<dyn GpuFuture>,
        groups: &[ModelDataGroup],
        base_matrix: Matrix4<f32>,
        data: &mut vs::ty::Data,
        command_buffer_builder: &mut AutoCommandBufferBuilder,
        pipeline: &mut RenderPipeline,
    ) {
        if !self.0.texture_future.read().is_empty() {
            let texture_futures =
                std::mem::replace(&mut *self.0.texture_future.write(), Vec::new());
            for fut in texture_futures {
                let tmp = std::mem::replace(future, now(pipeline.device.clone()).boxed());
                *future = tmp.join(fut).boxed();
            }
        }
        let layout = pipeline.gui_pipeline.descriptor_set_layout(0).unwrap();

        for (group, group_data) in self.0.groups.iter().zip(groups.iter()) {
            let texture = group
                .texture
                .as_ref()
                .unwrap_or(&pipeline.empty_texture)
                .clone();

            data.world = (base_matrix * group_data.matrix).into();
            crate::model::render::update_uniform_material(data, group.material.as_ref());
            let data = map_data(data);

            let uniform_buffer_subbuffer = pipeline.gui_uniform_buffer.next(data).unwrap();

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
                .or_else(|| self.0.vertex_buffer.as_ref())
                .expect("Model has no valid vertex buffer");

            if let Some(index) = group.index.as_ref() {
                command_buffer_builder
                    .draw_indexed(
                        pipeline.gui_pipeline.clone(),
                        &pipeline.dynamic_state,
                        vec![vertex_buffer.clone()],
                        index.clone(),
                        set.clone(),
                        (),
                    )
                    .unwrap();
            } else {
                command_buffer_builder
                    .draw(
                        pipeline.gui_pipeline.clone(),
                        &pipeline.dynamic_state,
                        vec![vertex_buffer.clone()],
                        set,
                        (),
                    )
                    .unwrap();
            }
        }
    }
}

fn map_data(d: &crate::model::vs::ty::Data) -> super::vs::ty::Data {
    super::vs::ty::Data {
        camera_x: d.camera_x,
        camera_y: d.camera_y,
        camera_z: d.camera_z,
        lightCount: d.lightCount,
        lights: map_lights(&d.lights),
        material_ambient_r: d.material_ambient_r,
        material_ambient_b: d.material_ambient_b,
        material_ambient_g: d.material_ambient_g,
        material_diffuse_r: d.material_diffuse_r,
        material_diffuse_b: d.material_diffuse_b,
        material_diffuse_g: d.material_diffuse_g,
        material_specular_r: d.material_specular_r,
        material_specular_b: d.material_specular_b,
        material_specular_g: d.material_specular_g,
        material_shininess: d.material_shininess,
        proj: d.proj,
        view: d.view,
        world: d.world,
    }
}

fn map_lights(
    lights: &[crate::model::vs::ty::DirectionalLight; 100],
) -> [super::vs::ty::DirectionalLight; 100] {
    array_init::array_init(|i| {
        let l = lights[i];
        super::vs::ty::DirectionalLight {
            color_ambient_r: l.color_ambient_r,
            color_ambient_g: l.color_ambient_g,
            color_ambient_b: l.color_ambient_b,
            color_diffuse_r: l.color_diffuse_r,
            color_diffuse_g: l.color_diffuse_g,
            color_diffuse_b: l.color_diffuse_b,
            color_specular_r: l.color_specular_r,
            color_specular_g: l.color_specular_g,
            color_specular_b: l.color_specular_b,
            direction_x: l.direction_x,
            direction_y: l.direction_y,
            direction_z: l.direction_z,
        }
    })
}

/*
impl GuiElement {
    pub fn new(device: Arc<Device>) -> Self {
        let points = &[
            Vertex {
                offset: [-0.5, -0.5],
                tex_coord: [0.0, 1.0],
            },
            Vertex {
                offset: [0.5, -0.5],
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                offset: [0.5, 0.5],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                offset: [-0.5, 0.5],
                tex_coord: [0.0, 0.0],
            },
        ];
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            points.into_iter().cloned(),
        )
        .unwrap();
        let index_buffer = CpuAccessibleBuffer::from_iter(
            device,
            BufferUsage::all(),
            false,
            indices.into_iter().cloned(),
        )
        .unwrap();

        Self {
            vertex_buffer,
            index_buffer,
        }
    }
    pub fn render(
        &self,
        fut: Box<dyn GpuFuture>,
        mut builder: AutoCommandBufferBuilder,
        pipeline: &mut RenderPipeline,
    ) -> (AutoCommandBufferBuilder, Box<dyn GpuFuture>) {
        let data = super::vs::ty::Data {
            position: [0.0, 0.0],
            size: [100.0, 100.0],
            screen_size: [800.0, 600.0],
        };

        let uniform_buffer_subbuffer = pipeline.gui_uniform_buffer.next(data).unwrap();
        let layout = pipeline.gui_pipeline.descriptor_set_layout(0).unwrap();
        let set = Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_buffer(uniform_buffer_subbuffer)
                .unwrap()
                .add_sampled_image(pipeline.empty_texture.clone(), pipeline.sampler.clone())
                .unwrap()
                .build_with_pool(&mut pipeline.descriptor_pool)
                .unwrap(),
        );

        builder = builder
            .draw_indexed(
                pipeline.gui_pipeline.clone(),
                &pipeline.dynamic_state,
                vec![self.vertex_buffer.clone()],
                self.index_buffer.clone(),
                set,
                (),
            )
            .unwrap();

        (builder, fut)
    }
}
*/
