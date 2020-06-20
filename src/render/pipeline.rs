use crate::{
    gui::Pipeline as GuiPipeline, model::Pipeline as ModelPipeline, state::InitError, GameState,
};
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

pub(crate) struct RenderPipeline {
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
    ) -> Result<Self, InitError> {
        let caps = surface
            .capabilities(physical)
            .map_err(InitError::CouldNotLoadSurfaceCapabilities)?;
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
            .unwrap(), // should never fail because the device should be valid and the parameters are hard-coded
        );

        let mut dynamic_state = DynamicState::none();

        let usage = caps.supported_usage_flags;
        let alpha = caps
            .supported_composite_alpha
            .iter()
            .next()
            .ok_or(InitError::NoCompositeAlpha)?;
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
        .map_err(InitError::CouldNotInitSwapchain)?;

        let framebuffers = Self::build_framebuffers(
            device.clone(),
            &swapchain_images,
            render_pass.clone(),
            &mut dynamic_state,
        )?;

        let descriptor_pool = Arc::new(StdDescriptorPool::new(device.clone()));

        let model_pipeline =
            ModelPipeline::create(device.clone(), queue.clone(), render_pass.clone());
        let gui_pipeline = GuiPipeline::create(device.clone(), render_pass.clone());
        Ok(Self {
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
        })
    }

    fn build_framebuffers(
        device: Arc<Device>,
        images: &[Arc<SwapchainImage<winit::window::Window>>],
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Result<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>, InitError> {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, dimensions[1] as f32],
            dimensions: [dimensions[0] as f32, -(dimensions[1] as f32)],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        let depth_buffer =
            AttachmentImage::transient(device, dimensions, Format::D16Unorm).unwrap(); // this should always be valid as long as the device is valid

        images
            .iter()
            .map(|image| {
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .and_then(|f| f.add(depth_buffer.clone()))
                    .and_then(|f| f.build())
                    .map(|fb| Arc::new(fb) as Arc<dyn FramebufferAbstract + Send + Sync>)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(InitError::CouldNotBuildSwapchainImages)
    }

    pub fn resize(&mut self, dimensions: [f32; 2]) {
        self.dimensions = dimensions;
        self.swapchain_needs_refresh = true;
    }

    fn get_swapchain_num(
        &mut self,
    ) -> Result<Option<(usize, SwapchainAcquireFuture<winit::window::Window>)>, InitError> {
        if self.swapchain_needs_refresh {
            let (new_swapchain, new_images) = match self
                .swapchain
                .recreate_with_dimensions([self.dimensions[0] as u32, self.dimensions[1] as u32])
            {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return Ok(None),
                Err(e) => return Err(InitError::CouldNotRecreateSwapchain(e)),
            };
            self.framebuffers = Self::build_framebuffers(
                self.device.clone(),
                &new_images,
                self.render_pass.clone(),
                &mut self.dynamic_state,
            )?;

            self.swapchain = new_swapchain;
            self.swapchain_images = new_images;
            self.swapchain_needs_refresh = false;
        }
        match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok((num, suboptimal, acquire_future)) => {
                if suboptimal {
                    self.swapchain_needs_refresh = true;
                }
                Ok(Some((num, acquire_future)))
            }
            Err(AcquireError::OutOfDate) => {
                self.swapchain_needs_refresh = true;
                self.get_swapchain_num()
            }
            Err(e) => Err(InitError::CouldNotAcquireSwapchainImage(e)),
        }
    }

    pub fn render(
        &mut self,
        dimensions: [f32; 2],
        game_state: &mut GameState,
    ) -> Result<Option<FenceSignalFuture<Box<dyn GpuFuture>>>, InitError> {
        let (image_num, acquire_future) = match self.get_swapchain_num()? {
            Some(r) => r,
            None => return Ok(None),
        };
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap(); // this can only throw an OomError, which we assume will not happen

        command_buffer_builder
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                false,
                vec![[0.5, 0.5, 1.0, 1.0].into(), 1f32.into()],
            )
            .unwrap(); // This can only error if we're in the wrong state of the command buffer, and the state is hard-coded

        // Build a list of futures that need to be processed before this frame is drawn
        let mut start_future = acquire_future.boxed();

        self.model_pipeline.render(
            &mut start_future,
            &mut command_buffer_builder,
            dimensions,
            game_state,
            &self.dynamic_state,
            &mut self.descriptor_pool,
        );

        for element in game_state.gui_elements.values_mut() {
            self.gui_pipeline.render_element(
                element,
                &mut command_buffer_builder,
                &mut start_future,
                self.dimensions,
                &self.dynamic_state,
                &mut self.descriptor_pool,
            );
        }

        command_buffer_builder.end_render_pass().unwrap(); // This can only error if we're in the wrong state of the command buffer, and the state is hard-coded

        let command_buffer = command_buffer_builder.build().unwrap(); // This can only error if we're in the wrong state, or we run out of memory

        let future = start_future
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap() // This error seems to never trigger
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .boxed();

        let future = future.then_signal_fence_and_flush();

        match future {
            Ok(f) => Ok(Some(f)),
            Err(e) => {
                if let FlushError::OutOfDate = e {
                    self.swapchain_needs_refresh = true;
                } else {
                    eprintln!("Failed to flush future: {:?}", e);
                }
                Ok(None)
            }
        }
    }

    pub fn finish_render(&mut self, future: Option<FenceSignalFuture<Box<dyn GpuFuture>>>) {
        if let Some(future) = future {
            future.wait(None).unwrap(); // This future seems to never fail
        }
    }
}
