use super::{fs, vs, GuiElementRef, Vertex};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    descriptor::descriptor_set::{PersistentDescriptorSet, StdDescriptorPool},
    device::Device,
    framebuffer::{RenderPassAbstract, Subpass},
    pipeline::{GraphicsPipeline, GraphicsPipelineAbstract},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    sync::{now, GpuFuture},
};

pub struct Pipeline {
    device: Arc<Device>,
    rect_vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
    rect_index: Arc<CpuAccessibleBuffer<[u16]>>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    sampler: Arc<Sampler>,
}

impl Pipeline {
    pub fn create(
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Self {
        // These should never fail, as the shaders are hard-coded and the device is assumed to be
        // valid.
        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .cull_mode_front()
                .blend_alpha_blending()
                .depth_stencil_simple_depth()
                // This should never fail because the render_pass is hard-coded
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                // This should never fail because all arguments are hard-coded
                .unwrap(),
        );
        let uniform_buffer = CpuBufferPool::<vs::ty::Data>::uniform_buffer(device.clone());

        let rect_vertex = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            VERTICES.iter().cloned(),
        )
        // This should never fail because the arguments are hard-coded
        .unwrap();
        let rect_index = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            INDICES.iter().cloned(),
        )
        // This should never fail because the arguments are hard-coded
        .unwrap();

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
        // This should never fail because the arguments are hard-coded
        .unwrap();

        Self {
            device,
            pipeline,
            uniform_buffer,
            rect_vertex,
            rect_index,
            sampler,
        }
    }
    pub fn render_element(
        &self,
        element: &mut GuiElementRef,
        command_buffer_builder: &mut AutoCommandBufferBuilder,
        future: &mut Box<dyn GpuFuture>,
        screen_size: [f32; 2],
        dynamic_state: &DynamicState,
        descriptor_pool: &mut Arc<StdDescriptorPool>,
    ) {
        if let Some(fut) = element.texture_future.take() {
            let tmp = std::mem::replace(future, now(self.device.clone()).boxed());
            *future = tmp.join(fut).boxed();
        }
        let element_data = element.data.read();
        let data = vs::ty::Data {
            screen_size,
            position: [
                element_data.dimensions.0 as f32,
                element_data.dimensions.1 as f32,
            ],
            size: [
                element_data.dimensions.2 as f32,
                element_data.dimensions.3 as f32,
            ],
        };
        // Should never fail if we have a valid uniform buffer
        let data = self.uniform_buffer.next(data).unwrap();

        // Should never fail because the pipeline and index are hard-coded
        let layout = self.pipeline.descriptor_set_layout(0).unwrap();
        let set = Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_buffer(data)
                // Should never fail because the layout and data are hard-coded
                .unwrap()
                .add_sampled_image(element.texture.clone(), self.sampler.clone())
                // Should never fail because the texture should be valid and the sampler is
                // hard-coded
                .unwrap()
                .build_with_pool(descriptor_pool)
                // Should never fail because if we have a valid descriptor_pool
                .unwrap(),
        );
        command_buffer_builder
            .draw_indexed(
                self.pipeline.clone(),
                dynamic_state,
                vec![self.rect_vertex.clone()],
                self.rect_index.clone(),
                set,
                (),
            )
            // Should never fail because we assume the command buffer is valid, the vertices and
            // indices are hard-coded, and the rest of the parameters are also valid
            .unwrap();
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        offset: [0.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    Vertex {
        offset: [0.0, 1.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        offset: [1.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        offset: [1.0, 1.0],
        tex_coord: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];
