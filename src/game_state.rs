use crate::{
    gui::{GuiElementBuilder, GuiElementRef},
    internal::UpdateMessage,
    model::{loader::ParsedModel, ModelBuilder, ModelRef, SourceOrShape},
    render::lights::LightState,
    state::GuiError,
    Font,
};
use cgmath::{Matrix4, SquareMatrix};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{mpsc::Sender, Arc},
    time::{Duration, Instant},
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
    pub(crate) model_handles: HashMap<u64, ModelRef>,
    pub(crate) internal_update_sender: Sender<UpdateMessage>,
    pub(crate) gui_elements: HashMap<u64, GuiElementRef>,
    pub(crate) is_running: bool,

    /// The matrix of the camera currently in use.
    ///
    /// It is currently not possible to change the near and far boundaries of the camera. This might be added in a later version.
    pub camera: Matrix4<f32>,

    /// Get the current keyboard state.
    pub keyboard: KeyboardState,

    /// The state of the lights currently in the world.
    pub light: LightState,

    /// The state of the time in the game. This is where you can get the `delta` time since the
    /// last frame.
    pub time: TimeState,

    surface: Arc<Surface<winit::window::Window>>,
}

impl GameState {
    pub(crate) fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        sender: Sender<UpdateMessage>,
        surface: Arc<Surface<winit::window::Window>>,
    ) -> Self {
        Self {
            device,
            queue,
            model_handles: HashMap::new(),
            internal_update_sender: sender,
            gui_elements: HashMap::new(),
            is_running: true,
            camera: Matrix4::identity(),
            keyboard: KeyboardState {
                pressed: HashSet::default(),
            },
            light: LightState::new(),
            time: TimeState::default(),
            surface,
        }
    }

    pub(crate) fn update(&mut self) {
        self.time.update();
    }

    /// Load a font from the given relative path. This function will panic if the font does not exist.
    ///
    /// The font is not stored internally, and must be stored by the developer.
    pub fn load_font(&mut self, font: impl AsRef<std::path::Path>) -> Result<Font, GuiError> {
        use std::{fs::File, io::Read};
        let font = font.as_ref();
        let font_str = font.to_str().unwrap_or("unknown");

        let mut file = File::open(font).map_err(|e| GuiError::CouldNotReadFontFile {
            file: font_str.to_string(),
            inner: e,
        })?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| GuiError::CouldNotReadFontFile {
                file: font_str.to_string(),
                inner: e,
            })?;

        match rusttype::Font::try_from_vec(content) {
            Some(font) => Ok(Arc::new(font)),
            None => Err(GuiError::CouldNotLoadFont),
        }
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
            .unwrap(); // we assume this always succeeds
    }

    /// Exit the game. Once this function is called, it cannot be cancelled. This does not confirm with [Game::can_shutdown](trait.Game.html#method.can_shutdown).
    pub fn terminate_game(&mut self) {
        self.is_running = false;
    }

    /// Get the width and height of the window, excluding the menu bar and borders. This is the renderable surface.
    ///
    /// This method is short for `window().inner_size()`
    pub fn window_size(&self) -> (u32, u32) {
        let size = self.window().inner_size();
        (size.width, size.height)
    }

    /// Create a new GUI element.
    /// The element will be placed at `dimensions.0 / dimensions.1` from the bottom-left of the window, with a size of `dimensions.2 x dimensions.3` scaling towards the top-right.
    /// The element will ignore window size, it is up to the developer to make sure elements are rendered inside of the window.
    ///
    /// The returned builder can either be turned into a [GuiElementTextureBuilder] by calling `.with_texture(path)`, or into a [GuiElementCanvasBuilder] by calling `.with_canvas(color)`.
    /// See the respective structs for more options.
    ///
    /// The returned [GuiElement] most be stored somewhere. When the GuiElement gets dropped, it will be removed from the screen.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use crystal_engine::*;
    /// # let mut state: GameState = unsafe { std::mem::zeroed() };
    /// let font = state.load_font("Roboto.ttf").unwrap(); // load the font. Make sure to store this somewhere.
    /// let text: GuiElement = state
    ///     .new_gui_element((100, 100, 300, 80)) // x, y, width, height of the element
    ///     .canvas() // Turn this into a white rectangle
    ///     .with_text(font.clone(), 32, "Hello world", color::BLACK) // with a black text
    ///     .with_border(3, color::BLACK) // and a black border
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// [GuiElementTextureBuilder]: ./state/struct.GuiElementTextureBuilder.html
    /// [GuiElementCanvasBuilder]: ./state/struct.GuiElementCanvasBuilder.html
    /// [GuiElement]: ./struct.GuiElement.html
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
    ///     .build()
    ///     .unwrap();
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
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// [ModelHandle]: ./struct.ModelHandle.html
    pub fn new_rectangle_model(&mut self) -> ModelBuilder {
        ModelBuilder::new(self, SourceOrShape::Rectangle)
    }

    /// Load a model externally. This allows you to define your own model loading, with more customization options.
    pub fn new_model(&mut self, parsed_model: ParsedModel) -> ModelBuilder {
        ModelBuilder::new(self, SourceOrShape::Custom(parsed_model))
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

/// The time state of the game. This contains all time-based values of the engine, like the `delta`
/// time since the last frame, the `running` time since the start of the game, and the `fps` of the
/// last 10 frames.
pub struct TimeState {
    start_instant: Instant,
    last_frame_instant: Instant,
    next_frame_instant: Instant,
    frame_times: VecDeque<Duration>,
}

const FRAME_TIME_COUNT: usize = 10;

impl Default for TimeState {
    fn default() -> Self {
        let instant = Instant::now();
        Self {
            start_instant: instant,
            last_frame_instant: instant,
            next_frame_instant: instant,
            frame_times: VecDeque::with_capacity(FRAME_TIME_COUNT),
        }
    }
}

impl TimeState {
    pub(crate) fn update(&mut self) {
        self.last_frame_instant = self.next_frame_instant;
        self.next_frame_instant = Instant::now();

        if self.frame_times.len() == FRAME_TIME_COUNT {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(self.delta());
    }

    /// Get the delta time since the last frame. This is used for consistent updates throughout the
    /// game where different screen refresh rates won't make objects move faster or slower.
    pub fn delta(&self) -> Duration {
        self.next_frame_instant - self.last_frame_instant
    }

    /// Get the total running time of the game. This is the time since the [GameState] has been
    /// created.
    pub fn running(&self) -> Duration {
        Instant::now() - self.start_instant
    }

    /// Get the average fps of the last 10 frames. This value will be `0.0` if no frames have been
    /// rendered yet.
    pub fn fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            0.0
        } else {
            let average_duration =
                self.frame_times.iter().sum::<Duration>() / (self.frame_times.len() as u32);
            1.0 / average_duration.as_secs_f32()
        }
    }
}

#[test]
fn test_timestate_never_resize() {
    let mut state = TimeState::default();
    let cap = state.frame_times.capacity();
    for _ in 0..cap * 10 {
        state.update();
        assert_eq!(cap, state.frame_times.capacity());
    }
    assert_eq!(FRAME_TIME_COUNT, state.frame_times.len());
}
