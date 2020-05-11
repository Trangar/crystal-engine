//! This is a prototype game engine, focussed on abstracting away all rendering logic and focussing purely on the game logic.
//!
//! # Example
//!
//! ```no_run
//! use cgmath::{Matrix4, Point3, Rad, Vector3};
//! use crystal_engine::{GameState, ModelHandle, Window, VirtualKeyCode};
//!
//! fn main() {
//!     // Create a new instance of your game and run it
//!     let window = Window::<Game>::new(800., 600.);
//!     window.run();
//! }
//!
//! pub struct Game {
//!     // Your game state is stored here
//!     model: ModelHandle,
//! }
//!
//! impl crystal_engine::Game for Game {
//!     fn init(state: &mut GameState) -> Self {
//!         // Load an object. This will automatically be rendered every frame
//!         // as long as the returned ModelHandle is not dropped.
//!         let model = state.create_model_from_obj("assets/some_object.obj");
//!
//!         // You can move the model around by calling `.modify`
//!         model.modify(|data| {
//!             data.position.y = -3.0;
//!             data.scale = 0.3;
//!         });
//!
//!         // Update the camera by manipulating the state's field
//!         state.camera = Matrix4::look_at(
//!             Point3::new(0.3, 0.3, 1.0),
//!             Point3::new(0.0, 0.0, 0.0),
//!             Vector3::new(0.0, -1.0, 0.0),
//!         );
//!
//!         Self { model }
//!     }
//!
//!     fn keydown(&mut self, state: &mut GameState, key: VirtualKeyCode) {
//!         // Exit the game when the user hits escape
//!         if key == VirtualKeyCode::Escape {
//!             state.terminate_game();
//!         }
//!     }
//!
//!     fn update(&mut self, state: &mut GameState) {
//!         self.model.modify(|data| {
//!             // Rotate either left or right, based on what the user has pressed
//!             if state.keyboard.is_pressed(VirtualKeyCode::A) {
//!                 data.rotation.y -= Rad(0.05);
//!             }
//!             if state.keyboard.is_pressed(VirtualKeyCode::D) {
//!                 data.rotation.y += Rad(0.05);
//!             }
//!         });
//!     }
//! }
//! ```

#![warn(missing_docs)]

mod game_state;
mod render;

pub use self::{
    game_state::GameState,
    render::{DirectionalLight, ModelData, ModelHandle, PointLight, PointLightAttenuation, Window},
};

/// Contains the states that are used in [GameState]. These are in a seperate module so we don't pollute the base module documentation.
pub mod state {
    pub use crate::{
        game_state::KeyboardState,
        render::{FixedVec, LightState},
    };
}

pub use winit::event::{VirtualKeyCode, WindowEvent};

/// The entry point of the game implementation.
///
/// In your game you will have to implement this trait for your own Game object. See the main module documentation for an example.
pub trait Game {
    /// Create a new instance of the game. This will be called exactly once, whenever the game window is created.
    fn init(state: &mut GameState) -> Self;
    /// Update the game. This will be called every frame. Use this to implement your game logic.
    fn update(&mut self, state: &mut GameState);
    /// Checks if the game can shut down. This is called when a player tries to close the window by clicking X or pressing alt+f4
    fn can_shutdown(&mut self, _state: &mut GameState) -> bool {
        true
    }
    /// Triggered when a winit event is received.
    fn event(&mut self, _state: &mut GameState, _event: &WindowEvent) {}
    /// Triggered when a key is pressed.
    ///
    /// Note that the [GameState.keyboard](struct.GameState.html#structfield.keyboard) is updated *before* this method is called.
    /// This means that `state.keyboard.is_pressed(key)` will always return `true`.
    fn keydown(&mut self, _state: &mut GameState, _key: VirtualKeyCode) {}
    /// Triggered when a key is released.
    ///
    /// Note that the [GameState.keyboard](struct.GameState.html#structfield.keyboard) is updated *before* this method is called.
    /// This means that `state.keyboard.is_pressed(key)` will always return `false`.
    fn keyup(&mut self, _state: &mut GameState, _key: VirtualKeyCode) {}
}
