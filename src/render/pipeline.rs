use crate::model::{fs as model_fs, vs as model_vs, ModelData};
use cgmath::{Matrix4, Rad, Zero};
use parking_lot::RwLock;
use std::{mem, sync::Arc};
use vulkano::{
    buffer::CpuBufferPool,
    command_buffer::{
        AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture, DynamicState,
    },
    descriptor::descriptor_set::StdDescriptorPool,
    device::{Device, Queue},
    format::{Format, R8G8B8A8Srgb},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{attachment::AttachmentImage, Dimensions, ImmutableImage, SwapchainImage},
    instance::PhysicalDevice,
    pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    swapchain::{
        AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
        Swapchain, SwapchainAcquireFuture, SwapchainCreationError,
    },
    sync::{FenceSignalFuture, FlushError, GpuFuture, NowFuture},
};

pub struct RenderPipeline {
    pub(crate) device: Arc<Device>,
    queue: Arc<Queue>,
    dimensions: [f32; 2],
    pub(crate) pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub(crate) dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    pub(crate) empty_texture: Arc<ImmutableImage<R8G8B8A8Srgb>>,
    pub(crate) render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub(crate) uniform_buffer: CpuBufferPool<model_vs::ty::Data>,
    swapchain: Arc<Swapchain<winit::window::Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
    swapchain_needs_refresh: bool,
    pub(crate) sampler: Arc<Sampler>,
    next_frame_futures: Vec<Box<dyn GpuFuture>>,

    pub(crate) descriptor_pool: Arc<StdDescriptorPool>,
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

        let vs = model_vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = model_fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<super::Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                //.cull_mode_front()
                .blend_alpha_blending()
                .depth_stencil_simple_depth()
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

        let framebuffers = Self::build_framebuffers(
            device.clone(),
            &swapchain_images,
            render_pass.clone(),
            &mut dynamic_state,
        );

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

        let (empty_texture, fut) = generate_empty_texture(queue.clone(), [255, 0, 0, 255]);
        let descriptor_pool = Arc::new(StdDescriptorPool::new(device.clone()));

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
            empty_texture,
            sampler,
            next_frame_futures: vec![Box::new(fut) as _],
            descriptor_pool,
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
        models: impl Iterator<Item = &'a Arc<RwLock<ModelData>>>,
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

        let proj = cgmath::perspective(
            Rad(std::f32::consts::FRAC_PI_2),
            dimensions[0] / dimensions[1],
            0.01,
            100.0,
        );
        let mut data = default_uniform(camera, proj, directional_lights);

        // Build a list of futures that need to be processed before this frame is drawn
        let mut start_future = Box::new(acquire_future) as Box<dyn GpuFuture>;
        // Drain the futures that were queued from last frame
        for fut in mem::replace(&mut self.next_frame_futures, Vec::new()) {
            start_future = Box::new(start_future.join(fut)) as _;
        }

        for handle in models {
            let handle = handle.read();

            handle.model.render(
                &mut start_future,
                &handle.groups,
                handle.matrix(),
                &mut data,
                &mut command_buffer_builder,
                self,
            );
        }
        command_buffer_builder.end_render_pass().unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();

        let future = Box::new(
            start_future
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                // The color output is now expected to contain our triangle. But in order to show it on
                // the screen, we have to *present* the image by calling `present`.
                //
                // This function does not actually present the image immediately. Instead it submits a
                // present command at the end of the queue. This means that it will only be presented once
                // the GPU has finished executing the command buffer that draws the triangle.
                .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num),
        ) as Box<dyn GpuFuture>;
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

fn default_uniform(
    camera: Matrix4<f32>,
    proj: Matrix4<f32>,
    directional_lights: (i32, [model_vs::ty::DirectionalLight; 100]),
) -> model_vs::ty::Data {
    let camera_pos = -camera.z.truncate();

    model_vs::ty::Data {
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
fn generate_empty_texture(
    queue: Arc<Queue>,
    color: [u8; 4],
) -> (
    Arc<ImmutableImage<R8G8B8A8Srgb>>,
    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
) {
    ImmutableImage::from_iter(
        color.iter().cloned(),
        Dimensions::Dim2d {
            width: 1,
            height: 1,
        },
        R8G8B8A8Srgb,
        queue,
    )
    .unwrap()
}
