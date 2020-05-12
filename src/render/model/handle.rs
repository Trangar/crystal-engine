use super::{Model, ModelData};
use cgmath::{Euler, Rad, Vector3};
use parking_lot::RwLock;
use std::sync::{mpsc::Sender, Arc};

/// A handle to the model that was loaded. This can be used to move the model around in the world.
///
/// When this handle is dropped, the model will disappear from the world on the next tick.
///
/// When this handle is cloned, a second model will appear in the world. Both models can be controlled independently.
pub struct ModelHandle {
    message_handle: Sender<ModelHandleMessage>,
    data: Arc<RwLock<ModelData>>,
}

impl ModelHandle {
    pub(crate) fn from_model(
        model: Arc<Model>,
        message_handle: Sender<ModelHandleMessage>,
    ) -> (Self, u64, Arc<RwLock<ModelData>>) {
        let (id, data) = ModelData::new(model);
        (
            Self {
                message_handle,
                data: data.clone(),
            },
            id,
            data,
        )
    }

    // TODO: Helper functions for:
    // - translate
    // - rotate_to
    // - rotate_by

    /// Get the current position of the handle. This is short for `self.read(|d| d.position)`
    pub fn position(&self) -> Vector3<f32> {
        self.read(|d| d.position)
    }

    /// Get the current rotation of the handle. This is short for `self.read(|d| d.rotation)`
    pub fn rotation(&self) -> Euler<Rad<f32>> {
        self.read(|d| d.rotation)
    }

    /// Get the current scale of the handle. This is short for `self.read(|d| d.scale)`
    pub fn scale(&self) -> f32 {
        self.read(|d| d.scale)
    }

    /// Read the data of the model. Optionally returning a value.
    ///
    /// ```no_run
    /// # use crystal_engine::*;
    /// # let mut game_state: GameState = unsafe { std::mem::zeroed() };
    /// let handle: ModelHandle = game_state.create_triangle();
    /// let scale = handle.read(|d| d.scale);
    /// ```
    pub fn read<T>(&self, cb: impl FnOnce(&ModelData) -> T) -> T {
        let data = self.data.read();
        cb(&data)
    }

    /// Update the model model. Optionally returning a value.
    ///
    /// ```no_run
    /// # use crystal_engine::*;
    /// # let mut game_state: GameState = unsafe { std::mem::zeroed() };
    /// let handle: ModelHandle = game_state.create_triangle();
    /// handle.modify(|d| d.scale = 0.0 );
    /// ```
    pub fn modify<T>(&self, cb: impl FnOnce(&mut ModelData) -> T) -> T {
        let mut data = self.data.write();
        cb(&mut data)
    }
}

impl Clone for ModelHandle {
    fn clone(&self) -> Self {
        let data = self.data.read();
        let model = data.model.clone();
        let (new_handle, new_id, new_data) =
            ModelHandle::from_model(model, self.message_handle.clone());

        {
            let mut new_data = new_data.write();
            new_data.position = data.position;
            new_data.rotation = data.rotation;
            new_data.scale = data.scale;
        }

        // This sender only errors when the receiver is dropped
        // which should only happen when the game is shutting down
        // so we ignore the error
        let _ = self
            .message_handle
            .send(ModelHandleMessage::NewClone(new_id, new_data));

        new_handle
    }
}

impl Drop for ModelHandle {
    fn drop(&mut self) {
        // This sender only errors when the receiver is dropped
        // which should only happen when the game is shutting down
        // so we ignore the error
        let _ = self
            .message_handle
            .send(ModelHandleMessage::Dropped(self.data.read().id));
    }
}

pub enum ModelHandleMessage {
    NewClone(u64, Arc<RwLock<ModelData>>),
    Dropped(u64),
}
