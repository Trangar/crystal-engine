use crate::{
    gui::{GuiElementRef, Pipeline as GuiPipeline},
    model::{vs as model_vs, ModelRef, Pipeline as ModelPipeline},
};
use cgmath::Matrix4;
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    descriptor::descriptor_set::StdDescriptorPool,
    device::{Device, Queue},
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::{attachment::AttachmentImage, SwapchainImage},
    instance::PhysicalDevice,
    pipeline::viewport::Viewport,
    swapchain::{
        AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
        Swapchain, SwapchainAcquireFuture, SwapchainCreationError,
    },
    sync::{FenceSignalFuture, FlushError, GpuFuture},
};

pub struct RenderPipeline {
    device: Arc<Device>,
    queue: Arc<Queue>,
    dimensions: [f32; 2],
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    swapchain: Arc<Swapchain<winit::window::Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
    swapchain_needs_refresh: bool,

    descriptor_pool: Arc<StdDescriptorPool>,
    model_pipeline: ModelPipeline,
    gui_pipeline: GuiPipeline,
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
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        );

        let mut dynamic_state = DynamicState::none();

        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

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

        let framebuffers = Self::build_framebuffers(
            device.clone(),
            &swapchain_images,
            render_pass.clone(),
            &mut dynamic_state,
        );

        let descriptor_pool = Arc::new(StdDescriptorPool::new(device.clone()));

        let model_pipeline =
            ModelPipeline::create(device.clone(), queue.clone(), render_pass.clone());
        let gui_pipeline = GuiPipeline::create(device.clone(), render_pass.clone());
        Self {
            device,
            queue,
            gui_pipeline,
            dynamic_state,
            framebuffers,
            render_pass,
            swapchain,
            swapchain_images,
            swapchain_needs_refresh: false,
            dimensions,
            descriptor_pool,
            model_pipeline,
        }
    }

    fn build_framebuffers(
        device: Arc<Device>,
        images: &[Arc<SwapchainImage<winit::window::Window>>],
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, dimensions[1] as f32],
            dimensions: [dimensions[0] as f32, -(dimensions[1] as f32)],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        let depth_buffer =
            AttachmentImage::transient(device, dimensions, Format::D16Unorm).unwrap();

        images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer.clone())
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
                self.device.clone(),
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

    pub fn render<'a>(
        &mut self,
        camera: Matrix4<f32>,
        dimensions: [f32; 2],
        models: impl Iterator<Item = &'a ModelRef>,
        gui_elements: impl Iterator<Item = &'a mut GuiElementRef>,
        directional_lights: (i32, [model_vs::ty::DirectionalLight; 100]),
    ) -> Option<FenceSignalFuture<Box<dyn GpuFuture>>> {
        let (image_num, acquire_future) = match self.get_swapchain_num() {
            Some(r) => r,
            None => return None,
        };
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap();

        command_buffer_builder
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                false,
                vec![[0.5, 0.5, 1.0, 1.0].into(), 1f32.into()],
            )
            .unwrap();

        // Build a list of futures that need to be processed before this frame is drawn
        let mut start_future = acquire_future.boxed();

        self.model_pipeline.render(
            &mut start_future,
            models,
            &mut command_buffer_builder,
            dimensions,
            camera,
            directional_lights,
            &self.dynamic_state,
            &mut self.descriptor_pool,
        );

        for element in gui_elements {
            self.gui_pipeline.render_element(
                element,
                &mut command_buffer_builder,
                &mut start_future,
                self.dimensions,
                &self.dynamic_state,
                &mut self.descriptor_pool,
            );
        }

        command_buffer_builder.end_render_pass().unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();

        let future = start_future
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .boxed();

        let future = future.then_signal_fence_and_flush();

        match future {
            Ok(f) => Some(f),
            Err(e) => {
                if let FlushError::OutOfDate = e {
                    self.swapchain_needs_refresh = true;
                } else {
                    eprintln!("Failed to flush future: {:?}", e);
                }
                None
            }
        }
    }

    pub fn finish_render(&mut self, future: Option<FenceSignalFuture<Box<dyn GpuFuture>>>) {
        if let Some(future) = future {
            future.wait(None).unwrap();
        }
    }
}
