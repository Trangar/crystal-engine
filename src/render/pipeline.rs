use super::model::{fs as model_fs, vs as model_vs, Model};
use cgmath::{Matrix4, Rad};
use std::sync::Arc;
use sync::FlushError;
use vulkano::{
    buffer::CpuBufferPool,
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    descriptor::descriptor_set::PersistentDescriptorSet,
    device::{Device, Queue},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::SwapchainImage,
    instance::PhysicalDevice,
    pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
    swapchain::{
        AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
        Swapchain, SwapchainAcquireFuture, SwapchainCreationError,
    },
    sync::{self, GpuFuture},
};

pub struct RenderPipeline {
    device: Arc<Device>,
    queue: Arc<Queue>,
    dimensions: [f32; 2],
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    uniform_buffer: CpuBufferPool<model_vs::ty::Data>,
    swapchain: Arc<Swapchain<winit::window::Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
    swapchain_needs_refresh: bool,
}

impl RenderPipeline {
    pub fn create(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface: Arc<Surface<winit::window::Window>>,
        physical: PhysicalDevice,
        dimensions: [f32; 2],
    ) -> Self {
        let caps = surface.capabilities(physical).unwrap();
        let format = caps.supported_formats[0].0;
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: format,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );

        let mut dynamic_state = DynamicState::none();

        let vs = model_vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = model_fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<super::Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let uniform_buffer = CpuBufferPool::<model_vs::ty::Data>::uniform_buffer(device.clone());

        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        println!("build_swapchain with dimensions {:?}", dimensions);

        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            surface,
            caps.min_image_count,
            format,
            [dimensions[0] as u32, dimensions[1] as u32],
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .unwrap();

        let framebuffers =
            Self::build_framebuffers(&swapchain_images, render_pass.clone(), &mut dynamic_state);

        Self {
            device,
            queue,
            pipeline,
            dynamic_state,
            framebuffers,
            render_pass,
            uniform_buffer,
            swapchain,
            swapchain_images,
            swapchain_needs_refresh: false,
            dimensions,
        }
    }

    fn build_framebuffers(
        images: &[Arc<SwapchainImage<winit::window::Window>>],
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>()
    }

    pub fn resize(&mut self, dimensions: [f32; 2]) {
        self.dimensions = dimensions;
        self.swapchain_needs_refresh = true;
    }

    fn get_swapchain_num(
        &mut self,
    ) -> Option<(usize, SwapchainAcquireFuture<winit::window::Window>)> {
        if self.swapchain_needs_refresh {
            let (new_swapchain, new_images) = match self
                .swapchain
                .recreate_with_dimensions([self.dimensions[0] as u32, self.dimensions[1] as u32])
            {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return None,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
            self.framebuffers = Self::build_framebuffers(
                &new_images,
                self.render_pass.clone(),
                &mut self.dynamic_state,
            );

            self.swapchain = new_swapchain;
            self.swapchain_images = new_images;
            self.swapchain_needs_refresh = false;
        }
        match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok((num, suboptimal, acquire_future)) => {
                if suboptimal {
                    self.swapchain_needs_refresh = true;
                }
                Some((num, acquire_future))
            }
            Err(AcquireError::OutOfDate) => {
                self.swapchain_needs_refresh = true;
                self.get_swapchain_num()
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        }
    }

    pub fn render(
        &mut self,
        camera: Matrix4<f32>,
        dimensions: [f32; 2],
        models: impl Iterator<Item = (Arc<Model>, Matrix4<f32>)>,
        callback: impl FnOnce(),
    ) {
        let (image_num, acquire_future) = match self.get_swapchain_num() {
            Some(r) => r,
            None => return,
        };
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .begin_render_pass(
            self.framebuffers[image_num].clone(),
            false,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .unwrap();

        let layout = self.pipeline.descriptor_set_layout(0).unwrap();

        let aspect_ratio = dimensions[0] / dimensions[1];
        let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);

        for (model, matrix) in models {
            let data = model_vs::ty::Data {
                world: matrix.into(),
                view: camera.into(),
                proj: proj.into(),
            };
            let uniform_buffer_subbuffer = self.uniform_buffer.next(data).unwrap();

            let set = Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            if let Some(indices) = model.indices.as_ref() {
                command_buffer_builder = command_buffer_builder
                    .draw_indexed(
                        self.pipeline.clone(),
                        &self.dynamic_state,
                        vec![model.vertex_buffer.clone()],
                        indices.clone(),
                        set,
                        (),
                    )
                    .unwrap();
            } else {
                command_buffer_builder = command_buffer_builder
                    .draw(
                        self.pipeline.clone(),
                        &self.dynamic_state,
                        vec![model.vertex_buffer.clone()],
                        set,
                        (),
                    )
                    .unwrap();
            }
        }

        let command_buffer = command_buffer_builder
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        let future = acquire_future
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        callback();

        if let Err(e) = future {
            if let FlushError::OutOfDate = e {
                self.swapchain_needs_refresh = true;
            } else {
                eprintln!("Failed to flush future: {:?}", e);
            }
        }
    }
}
