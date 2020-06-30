use super::pipeline::RenderPipeline;
use crate::{internal::UpdateMessage, state::InitError, Game, GameState};
use std::sync::mpsc::{channel, Receiver};
use vulkano::{
    device::{Device, DeviceExtensions, Features},
    instance::{
        debug::{DebugCallback, MessageSeverity},
        Instance, InstanceExtensions, PhysicalDevice, QueueFamily, Version,
    },
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// A handle to the window and the game state. This will be your main entrypoint of the game.
pub struct Window<GAME: Game + 'static> {
    pipeline: RenderPipeline,
    events_loop: EventLoop<()>,
    state: WindowState<GAME>,
}

struct WindowState<GAME: Game + 'static> {
    dimensions: [f32; 2],
    game_state: GameState,
    model_handle_receiver: Receiver<UpdateMessage>,
    game: GAME,
    _dbg: Option<DebugCallback>,
}

fn msg_severity(s: MessageSeverity) -> char {
    if s.error {
        'E'
    } else if s.warning {
        'W'
    } else if s.information {
        'I'
    } else if s.verbose {
        'V'
    } else {
        '?'
    }
}

impl<GAME: Game + 'static> Window<GAME> {
    /// Create a new instance of the window. This will immediately instantiate an instance of [Game].
    pub fn new(width: f32, height: f32) -> Result<Self, InitError> {
        let instance = {
            let extensions = InstanceExtensions {
                ext_debug_utils: true,
                ..vulkano_win::required_extensions()
            };
            Instance::new(None, &extensions, None).map_err(InitError::CouldNotInitVulkano)?
        };

        let _dbg = if cfg!(debug_assertions) {
            DebugCallback::errors_and_warnings(&instance, |msg| {
                println!("{}> {}", msg_severity(msg.severity), msg.description);
            })
            .ok()
        } else {
            None
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
                        .ok_or(InitError::CouldNotFindValidGraphicsQueue)?,
                );
                true
            } else {
                false
            };
            print_physical_device_info(&device, picked, if picked { queue_family } else { None });
        }
        let physical = physical.ok_or(InitError::CouldNotFindPhysicalDevice)?;
        let queue_family = queue_family.ok_or(InitError::CouldNotFindValidGraphicsQueue)?;

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
            .map_err(InitError::CouldNotCreateDevice)?;
            (
                device,
                queues
                    .next()
                    .ok_or(InitError::CouldNotFindValidGraphicsQueue)?,
            )
        };
        let events_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .map_err(InitError::CouldNotCreateWindow)?;

        let pipeline = RenderPipeline::create(
            device.clone(),
            queue.clone(),
            surface.clone(),
            physical,
            [width, height],
        )?;

        let (sender, receiver) = channel();

        let mut game_state = GameState::new(device, queue, sender, surface);

        let game = GAME::init(&mut game_state);

        Ok(Window {
            pipeline,
            events_loop,
            state: WindowState {
                dimensions: [width, height],
                model_handle_receiver: receiver,
                game_state,
                game,
                _dbg,
            },
        })
    }

    /// Take control of the main loop and run the game. Periodically [Game::update] will be called, allowing you to modify the game world.
    pub fn run(self) -> ! {
        let Window {
            events_loop,
            mut pipeline,
            mut state,
        } = self;
        events_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(newsize),
                    ..
                } => {
                    state.dimensions = [newsize.width as f32, newsize.height as f32];
                    pipeline.resize(state.dimensions);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } if state.game.can_shutdown(&mut state.game_state) => {
                    *control_flow = ControlFlow::Exit
                }
                Event::RedrawEventsCleared => {
                    match pipeline.render(state.dimensions, &mut state.game_state) {
                        Err(e) => {
                            eprintln!("Engine encountered a fatal error");
                            eprintln!();
                            eprintln!("{:?}", e);
                            eprintln!();
                            eprintln!("Exiting now");
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        Ok(future) => {
                            state.update();
                            pipeline.finish_render(future);
                        }
                    }
                }
                _ => {}
            }
            if let Event::WindowEvent { event, .. } = event {
                state.game.event(&mut state.game_state, &event);
                if let WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: keystate,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } = event
                {
                    if keystate == ElementState::Pressed {
                        state.game_state.keyboard.pressed.insert(key);
                        state.game.keydown(&mut state.game_state, key);
                    } else {
                        state.game_state.keyboard.pressed.remove(&key);
                        state.game.keyup(&mut state.game_state, key);
                    }
                }
            }

            if !state.game_state.is_running {
                *control_flow = ControlFlow::Exit;
            }
        });
    }
}

impl<GAME: Game + 'static> WindowState<GAME> {
    fn update(&mut self) {
        self.game_state.update();
        self.game.update(&mut self.game_state);

        while let Ok(msg) = self.model_handle_receiver.try_recv() {
            msg.apply(&mut self.game_state);
        }
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
