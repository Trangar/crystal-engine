//! Contains information that is relevant to events coming from the system, e.g. keyboard input.

use crate::{Game, GameState};
use winit::event::{ElementState, KeyboardInput};

pub use winit::{dpi::PhysicalPosition, event::*};

// TODO: Move all event state (e.g. keyboard pressed keys) to this state
// This should probably be a field on GameState
pub(crate) struct EventState {}

impl EventState {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn update<GAME: Game>(
        &mut self,
        event: WindowEvent,
        game: &mut GAME,
        game_state: &mut GameState,
    ) {
        game.event(game_state, &event);
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
                game_state.keyboard.pressed.insert(key);
                game.keydown(game_state, key);
            } else {
                game_state.keyboard.pressed.remove(&key);
                game.keyup(game_state, key);
            }
        }
    }
}
