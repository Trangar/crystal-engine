//! Contains information that is relevant to events coming from the system, e.g. keyboard input.

use crate::{Game, GameState};

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

        if let WindowEvent::CursorMoved { position, .. } = event {
            let new_pos: (f32, f32) = position.into();
            let diff = (
                new_pos.0 - game_state.mouse.position.0,
                new_pos.1 - game_state.mouse.position.1,
            );
            game_state.mouse.position = new_pos;
            game.mouse_moved(game_state, diff);
        }

        if let WindowEvent::MouseInput { button, state, .. } = event {
            match button {
                MouseButton::Left => {
                    let was_pressed = game_state.mouse.left_pressed;
                    game_state.mouse.left_pressed = state == ElementState::Pressed;
                    if !was_pressed && game_state.mouse.left_pressed {
                        if let Some(id) = game_state.gui_element_id_at(game_state.mouse.position) {
                            game.gui_element_clicked(game_state, id);
                        }
                    }
                }
                MouseButton::Middle => {
                    game_state.mouse.middle_pressed = state == ElementState::Pressed;
                }
                MouseButton::Right => {
                    game_state.mouse.right_pressed = state == ElementState::Pressed;
                }
                MouseButton::Other(_) => {}
            }
        }
    }
}
