use crate::render::{Model, ModelData, ModelHandle, ModelHandleMessage};
use cgmath::{Matrix4, SquareMatrix};
use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::Sender, Arc},
};
use vulkano::device::Device;
use winit::event::VirtualKeyCode;

/// Contains the game state. This struct is passed to [Game::init](trait.Game.html#tymethod.init) and [Game::update](trait.Game.html#tymethod.update).
pub struct GameState {
    device: Arc<Device>,
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
}

impl GameState {
    pub(crate) fn new(device: Arc<Device>, sender: Sender<ModelHandleMessage>) -> Self {
        Self {
            device,
            model_handles: HashMap::new(),
            model_handle_sender: sender,
            is_running: true,
            // models: Vec::new(),
            camera: Matrix4::identity(),
            keyboard: KeyboardState {
                pressed: HashSet::default(),
            },
        }
    }

    pub(crate) fn add_model_data(&mut self, new_id: u64, handle: Arc<RwLock<ModelData>>) {
        self.model_handles.insert(new_id, handle);
    }

    pub(crate) fn remove_model_handle(&mut self, handle: u64) {
        self.model_handles.remove(&handle);
    }

    /// Exit the game. Once this function is called, it cannot be cancelled. This does not confirm with [Game::can_shutdown](trait.Game.html#method.can_shutdown).
    pub fn terminate_game(&mut self) {
        self.is_running = false;
    }

    /// Create a new triangle at the origin of the world.
    /// See [ModelHandle] for information on how to move and rotate the triangle.
    ///
    /// To create a second instance with the same model, simply call [ModelHandle::clone](struct.ModelHandle.html#impl-Clone)
    pub fn create_triangle(&mut self) -> ModelHandle {
        let model = Model::new_triangle(self.device.clone());
        self.add_model(model)
    }

    /// Create a new square at the origin of the world. This can be useful to render simple
    /// textures in the world.
    ///
    /// See [ModelHandle] for information on how to move and rotate the triangle.
    ///
    /// To create a second instance with the same model, simply call [ModelHandle::clone](struct.ModelHandle.html#impl-Clone)
    pub fn create_square(&mut self) -> ModelHandle {
        let model = Model::new_square(self.device.clone());
        self.add_model(model)
    }

    /// Load a model from the given path and place it at the origin of the world.
    /// See [ModelHandle] for information on how to move and rotate the triangle.
    ///
    /// To create a second instance with the same model, simply call [ModelHandle::clone](struct.ModelHandle.html#impl-Clone)
    pub fn create_model_from_obj(&mut self, path: impl AsRef<std::path::Path>) -> ModelHandle {
        let model = Model::from_obj_file(self.device.clone(), path);
        self.add_model(model)
    }

    fn add_model(&mut self, model: Arc<Model>) -> ModelHandle {
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
