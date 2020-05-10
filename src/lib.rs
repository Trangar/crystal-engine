//! This is a prototype game engine, focussed on abstracting away all rendering logic and focussing purely on the game logic.
//!
//! # Example
//!
//! ```no_run
//! use cgmath::{Matrix4, Point3, Rad, Vector3};
//! use crystal_engine::{GameState, ModelHandle, Window};
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
//!     fn update(&mut self, _state: &mut GameState) {
//!         // This will make our model spin
//!         self.model.modify(|data| {
//!             data.rotation.y += Rad(0.05);
//!         });
//!     }
//! }
//! ```

#![warn(missing_docs, clippy::broken_links)]

mod game_state;
mod render;

pub use self::{
    game_state::GameState,
    render::{ModelHandle, Window},
};

pub use winit::event::{VirtualKeyCode, WindowEvent};

/// The entry point of the game implementation.
pub trait Game {
    /// Create a new instance of the game
    fn init(state: &mut GameState) -> Self;
    /// Checks if the game can shut down. This is called when a player tries to close the window by clicking X or pressing alt+f4
    fn can_shutdown(&mut self, _state: &mut GameState) -> bool {
        true
    }
    /// Triggered when a winit event is received.
    fn event(&mut self, _state: &mut GameState, _event: &WindowEvent) {}
    /// Triggered when a key is pressed
    fn keydown(&mut self, _state: &mut GameState, _key: VirtualKeyCode) {}
    /// Triggered when a key is released
    fn keyup(&mut self, _state: &mut GameState, _key: VirtualKeyCode) {}
    /// Update the game
    fn update(&mut self, state: &mut GameState);
}
