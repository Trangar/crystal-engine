use crate::{
    gui::{GuiElementBuilder, GuiElementRef},
    model::{ModelBuilder, ModelData, ModelHandleMessage, SourceOrShape},
    render::LightState,
};
use cgmath::{Matrix4, SquareMatrix};
use parking_lot::RwLock;
use rusttype::Font;
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
    pub(crate) model_handle_sender: Sender<ModelHandleMessage>,
    pub(crate) gui_elements: Vec<GuiElementRef>,
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
            gui_elements: Vec::new(),
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

    pub fn load_font(&mut self, font: impl AsRef<std::path::Path>) -> Font<'static> {
        use std::{fs::File, io::Read};

        let mut file = File::open(font.as_ref()).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();

        Font::try_from_vec(content).unwrap()
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
    /// ```rust, no_run
    /// # use crystal_engine::*;
    /// # let state: GameState = unsafe { std::mem::zeroed() };
    /// state.window()
    ///      .set_cursor_position(winit::dpi::PhysicalPosition::new(0u32, 0u32))
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

    /// Create a new GUI element
    ///
    /// TODO: Example
    pub fn new_gui_element(&mut self, dimensions: (i32, i32, u32, u32)) -> GuiElementBuilder {
        GuiElementBuilder::new(self, dimensions)
    }

    /// Create a new triangle at the origin of the world.
    ///
    /// See [ModelHandle] for information on how to move, rotate and clone the triangle.
    ///
    /// Note: you *must* store the handle somewhere. When the handle is dropped, the rectangle is removed from your world and resources are unloaded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use crystal_engine::*;
    /// # let mut game_state: GameState = unsafe { std::mem::zeroed() };
    /// let triangle: ModelHandle = game_state.new_triangle_model()
    ///     .build();
    /// ```
    /// [ModelHandle]: ./struct.ModelHandle.html
    pub fn new_triangle_model(&mut self) -> ModelBuilder {
        ModelBuilder::new(self, SourceOrShape::Triangle)
    }

    /// Create a new rectangle at the origin of the world. This can be useful to render simple
    /// textures in the world.
    ///
    /// See [ModelHandle] for information on how to move, rotate and clone the rectangle.
    ///
    /// Note: you *must* store the handle somewhere. When the handle is dropped, the rectangle is removed from your world and resources are unloaded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use crystal_engine::*;
    /// # let mut game_state: GameState = unsafe { std::mem::zeroed() };
    /// let rust_logo: ModelHandle = game_state.new_rectangle_model()
    ///     .with_texture_from_file("assets/rust_logo.png")
    ///     .build();
    /// ```
    ///
    /// [ModelHandle]: ./struct.ModelHandle.html
    pub fn new_rectangle_model(&mut self) -> ModelBuilder {
        ModelBuilder::new(self, SourceOrShape::Rectangle)
    }

    #[cfg(feature = "format-obj")]
    /// Load a model from the given path and place it at the origin of the world.
    /// See [ModelHandle] for information on how to move, rotate and clone the model.
    ///
    /// This method is only available when the `format-obj` feature is enabled.
    ///
    /// [ModelHandle]: ./struct.ModelHandle.html
    pub fn new_obj_model<'a>(&'a mut self, path: &'a str) -> ModelBuilder<'a> {
        ModelBuilder::new(self, SourceOrShape::Obj(path))
    }

    #[cfg(feature = "format-fbx")]
    /// Load a model from the given path and place it at the origin of the world.
    /// See [ModelHandle] for information on how to move, rotate and clone the model.
    ///
    /// This method is only available when the `format-fbx` feature is enabled.
    ///
    /// [ModelHandle]: ./struct.ModelHandle.html
    pub fn new_fbx_model<'a>(&'a mut self, path: &'a str) -> ModelBuilder<'a> {
        ModelBuilder::new(self, SourceOrShape::Fbx(path))
    }
}

/// The state of the keyboard. This can be used to check which keys are pressed during the current frame.
///
/// Note: when implementing [Game] and handling `keydown` or `keyup`, the [GameState] will be updated *before* the keydown method is called.
///
/// [GameState]: ../struct.GameState.html
/// [Game]: ../trait.Game.html
pub struct KeyboardState {
    pub(crate) pressed: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    /// Check if the given key is pressed.
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed.contains(&key)
    }
}
