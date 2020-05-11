use super::{model_handle::ModelHandleMessage, RenderPipeline};
use crate::{Game, GameState};
use std::sync::mpsc::{channel, Receiver};
use vulkano::{
    device::{Device, DeviceExtensions, Features},
    instance::{Instance, PhysicalDevice, QueueFamily, Version},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// A handle to the window and the game state. This will be your main entrypoint of the game.
pub struct Window<GAME: Game + 'static> {
    dimensions: [f32; 2],
    pipeline: Option<RenderPipeline>,
    events_loop: Option<EventLoop<()>>,
    game_state: GameState,
    model_handle_receiver: Receiver<ModelHandleMessage>,
    game: GAME,
}

impl<GAME: Game + 'static> Window<GAME> {
    /// Create a new instance of the window. This will immediately instantiate an instance of [Game].
    pub fn new(width: f32, height: f32) -> Self {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
        };

        let mut physical = None;
        let mut queue_family = None;
        for device in PhysicalDevice::enumerate(&instance) {
            let picked = if physical.is_none() {
                physical = Some(device);
                queue_family = Some(
                    device
                        .queue_families()
                        .find(|q| q.supports_graphics())
                        .unwrap(),
                );
                true
            } else {
                false
            };
            print_physical_device_info(&device, picked, if picked { queue_family } else { None });
        }
        let physical = physical.expect("no device available");
        let queue_family = queue_family.expect("Couldn't find a graphical queue family");

        let (device, queue) = {
            let (device, mut queues) = Device::new(
                physical,
                &Features::none(),
                &DeviceExtensions {
                    khr_storage_buffer_storage_class: true,
                    khr_swapchain: true,
                    ..DeviceExtensions::none()
                },
                [(queue_family, 0.5)].iter().cloned(),
            )
            .expect("Could not create device");
            (device, queues.next().unwrap())
        };
        let events_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        let pipeline =
            RenderPipeline::create(device.clone(), queue, surface, physical, [width, height]);

        let (sender, receiver) = channel();

        let mut game_state = GameState::new(device, sender);

        let game = GAME::init(&mut game_state);

        Window {
            dimensions: [width, height],
            pipeline: Some(pipeline),
            events_loop: Some(events_loop),
            model_handle_receiver: receiver,
            game_state,
            game,
        }
    }

    pub(crate) fn update_size(&mut self, width: f32, height: f32) {
        self.dimensions = [width, height];
        self.pipeline.as_mut().unwrap().resize(self.dimensions);
    }

    fn update(&mut self) {
        self.game.update(&mut self.game_state);

        while let Ok(msg) = self.model_handle_receiver.try_recv() {
            match msg {
                ModelHandleMessage::Dropped(id) => self.game_state.remove_model_handle(id),
                ModelHandleMessage::NewClone(new_id, data) => {
                    self.game_state.add_model_data(new_id, data)
                }
            }
        }
    }

    fn render_and_update(&mut self) {
        let mut pipeline = self.pipeline.take().unwrap();
        let dimensions = self.dimensions;
        let handles: Vec<_> = self
            .game_state
            .model_handles
            .values()
            .map(|handle| {
                let handle = handle.read();
                (handle.model.clone(), handle.matrix())
            })
            .collect();

        pipeline.render(
            self.game_state.camera,
            dimensions,
            handles.into_iter(),
            self.game_state.light.directional.to_shader_value(),
            || {
                self.update();
            },
        );
        self.pipeline = Some(pipeline);
    }

    /// Take control of the main loop and run the game. Periodically [Game::update] will be called, allowing you to modify the game world.
    pub fn run(mut self) {
        let events_loop = self.events_loop.take().unwrap();
        events_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(newsize),
                    ..
                } => {
                    self.update_size(newsize.width as f32, newsize.height as f32);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } if self.game.can_shutdown(&mut self.game_state) => {
                    *control_flow = ControlFlow::Exit
                }
                Event::RedrawEventsCleared => {
                    self.render_and_update();
                }
                _ => {}
            }
            if let Event::WindowEvent { event, .. } = event {
                self.game.event(&mut self.game_state, &event);
                if let WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } = event
                {
                    if state == ElementState::Pressed {
                        self.game_state.keyboard.pressed.insert(key);
                        self.game.keydown(&mut self.game_state, key);
                    } else {
                        self.game_state.keyboard.pressed.remove(&key);
                        self.game.keyup(&mut self.game_state, key);
                    }
                }
            }

            if !self.game_state.is_running {
                *control_flow = ControlFlow::Exit;
            }
        });
    }
}

fn print_physical_device_info(
    device: &PhysicalDevice,
    picked: bool,
    queue_family: Option<QueueFamily>,
) {
    println!(
        "{} {}",
        if picked { "\u{2192}" } else { "-" },
        device.name(),
    );
    println!("  - api verison: {}", device.api_version());
    println!(
        "  - driver version: {} (0x{:08X})",
        Version::from_vulkan_version(device.driver_version()),
        device.driver_version()
    );
    println!("  Queue families:");
    for family in device.queue_families() {
        let picked = queue_family.as_ref() == Some(&family);
        println!(
            "  {} {}, queue count: {:2}, graphics: {:5}, compute: {:5}",
            if picked { "\u{2192}" } else { "-" },
            family.id(),
            family.queues_count(),
            family.supports_graphics(),
            family.supports_compute(),
        );
    }
}
