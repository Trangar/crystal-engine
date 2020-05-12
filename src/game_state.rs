use crate::render::{
    LightState, Model, ModelBuilder, ModelData, ModelHandle, ModelHandleMessage, SourceOrShape,
};
use cgmath::{Matrix4, SquareMatrix};
use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::Sender, Arc},
};
use vulkano::{
    device::{Device, Queue},
    swapchain::Surface,
};
use winit::event::VirtualKeyCode;

/// Contains the game state. This struct is passed to [Game::init](trait.Game.html#tymethod.init) and [Game::update](trait.Game.html#tymethod.update).
pub struct GameState {
    pub(crate) device: Arc<Device>,
    pub(crate) queue: Arc<Queue>,
    // models: Vec<Arc<Model>>,
    pub(crate) model_handles: HashMap<u64, Arc<RwLock<ModelData>>>,
    model_handle_sender: Sender<ModelHandleMessage>,
    pub(crate) is_running: bool,
    /// The matrix of the camera currently in use.
    ///
    /// It is currently not possible to change the near and far boundaries of the camera. This might be added in a later version.
    pub camera: Matrix4<f32>,
    /// Get the current keyboard state.
    pub keyboard: KeyboardState,
    /// The state of the lights currently in the world.
    pub light: LightState,
    surface: Arc<Surface<winit::window::Window>>,
}

impl GameState {
    pub(crate) fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        sender: Sender<ModelHandleMessage>,
        surface: Arc<Surface<winit::window::Window>>,
    ) -> Self {
        Self {
            device,
            queue,
            model_handles: HashMap::new(),
            model_handle_sender: sender,
            is_running: true,
            // models: Vec::new(),
            camera: Matrix4::identity(),
            keyboard: KeyboardState {
                pressed: HashSet::default(),
            },
            light: LightState::new(),
            surface,
        }
    }

    pub(crate) fn add_model_data(&mut self, new_id: u64, handle: Arc<RwLock<ModelData>>) {
        self.model_handles.insert(new_id, handle);
    }

    pub(crate) fn remove_model_handle(&mut self, handle: u64) {
        self.model_handles.remove(&handle);
    }

    /// Get a reference to the winit window. This can be used to set the title with `set_title`, grap the cursor with `set_cursor_grab` and `set_cursor_visible`, and more.
    pub fn window(&self) -> &winit::window::Window {
        self.surface.window()
    }

    /// Set the cursor position. This is short for:
    ///
    /// ```rust
    /// state.window()
    ///      .set_cursor_position(winit::dpi::PhysicalPosition::new(position.0, position.1))
    ///      .unwrap();
    /// ```
    ///
    /// # Platform-specific
    ///
    /// On iOS this always returns an error, so this function is empty
    #[cfg(target = "ios")]
    pub fn set_cursor_position<P: winit::dpi::Pixel>(&self, _position: (P, P)) {}

    /// Set the cursor position. This is short for:
    ///
    /// ```rust
    /// state.window()
    ///      .set_cursor_position(winit::dpi::PhysicalPosition::new(position.0, position.1))
    ///      .unwrap();
    /// ```
    ///
    /// # Platform-specific
    ///
    /// On iOS this always returns an error, so this function is empty
    #[cfg(not(target = "ios"))]
    pub fn set_cursor_position<P: winit::dpi::Pixel>(&self, position: (P, P)) {
        self.window()
            .set_cursor_position(winit::dpi::PhysicalPosition::new(position.0, position.1))
            .unwrap();
    }

    /// Exit the game. Once this function is called, it cannot be cancelled. This does not confirm with [Game::can_shutdown](trait.Game.html#method.can_shutdown).
    pub fn terminate_game(&mut self) {
        self.is_running = false;
    }

    /// Create a new triangle at the origin of the world.
    /// See [ModelHandle] for information on how to move and rotate the triangle.
    ///
    /// To create a second instance with the same model, simply call [ModelHandle::clone](struct.ModelHandle.html#impl-Clone)
    pub fn new_triangle(&mut self) -> ModelBuilder {
        ModelBuilder::new(self, SourceOrShape::Triangle)
    }

    /// Create a new rectangle at the origin of the world. This can be useful to render simple
    /// textures in the world.
    ///
    /// See [ModelHandle] for information on how to move and rotate the triangle.
    ///
    /// To create a second instance with the same model, simply call [ModelHandle::clone](struct.ModelHandle.html#impl-Clone)
    pub fn new_rectangle(&mut self) -> ModelBuilder {
        ModelBuilder::new(self, SourceOrShape::Rectangle)
    }

    /// Load a model from the given path and place it at the origin of the world.
    /// See [ModelHandle] for information on how to move and rotate the triangle.
    ///
    /// To create a second instance with the same model, simply call [ModelHandle::clone](struct.ModelHandle.html#impl-Clone)
    pub fn new_model_from_obj<'a>(&'a mut self, path: &'a str) -> ModelBuilder<'a> {
        ModelBuilder::new(self, SourceOrShape::Source(path))
    }

    pub(crate) fn add_model(&mut self, model: Arc<Model>) -> ModelHandle {
        let (handle, id, data) = ModelHandle::from_model(model, self.model_handle_sender.clone());
        // self.models.push(model);
        self.model_handles.insert(id, data);
        handle
    }
}

/// The state of the keyboard. This can be used to check which keys are pressed during the current frame.
///
/// Note: when handling [Game::keydown](../trait.Game.html#method.keydown), the [GameState] will be updated *before* the keydown method is called.
pub struct KeyboardState {
    pub(crate) pressed: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    /// Check if the given key is pressed.
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed.contains(&key)
    }
}
