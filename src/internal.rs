use crate::{gui::GuiElementData, GameState, ModelData};
use parking_lot::RwLock;
use std::sync::Arc;

pub enum UpdateMessage {
    NewModel {
        old_id: u64,
        new_id: u64,
        data: Arc<RwLock<ModelData>>,
    },
    ModelDropped(u64),
    NewGuiElement {
        old_id: u64,
        new_id: u64,
        data: Arc<RwLock<GuiElementData>>,
    },
    GuiElementDropped(u64),
}

impl UpdateMessage {
    pub fn apply(self, game_state: &mut GameState) {
        match self {
            UpdateMessage::ModelDropped(id) => {
                game_state.model_handles.remove(&id);
            }
            UpdateMessage::NewModel {
                old_id,
                new_id,
                data,
            } => {
                let old = &game_state.model_handles[&old_id];
                let new = old.with_new_data(data);
                game_state.model_handles.insert(new_id, new);
            }
            UpdateMessage::GuiElementDropped(id) => {
                game_state.gui_elements.remove(&id);
            }
            UpdateMessage::NewGuiElement {
                old_id,
                new_id,
                data,
            } => {
                let old = &game_state.gui_elements[&old_id];
                let new = old.with_new_data(data);
                game_state.gui_elements.insert(new_id, new);
            }
        }
    }
}
