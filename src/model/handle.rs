use super::{Model, ModelData, ModelDataGroup};
use crate::internal::UpdateMessage;
use cgmath::{Euler, Rad, Vector3};
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::Sender,
    Arc,
};

static ID: AtomicU64 = AtomicU64::new(1);

/// A handle to the model that was loaded. This can be used to move the model around in the world.
///
/// When this handle is dropped, the model will disappear from the world on the next tick.
///
/// When this handle is cloned, a second model will appear in the world. Both models can be controlled independently.
pub struct ModelHandle {
    id: u64,
    message_handle: Sender<UpdateMessage>,
    data: Arc<RwLock<ModelData>>,
}

impl ModelHandle {
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
    /// let handle: ModelHandle = game_state.new_triangle_model().build();
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
    /// let handle: ModelHandle = game_state.new_triangle_model().build();
    /// handle.modify(|d| d.scale = 0.0 );
    /// ```
    pub fn modify<T>(&self, cb: impl FnOnce(&mut ModelData) -> T) -> T {
        let mut data = self.data.write();
        cb(&mut data)
    }
}

impl Clone for ModelHandle {
    fn clone(&self) -> Self {
        let new_id = ID.fetch_add(1, Ordering::Relaxed);
        let message_handle = self.message_handle.clone();
        let data = self.data.read();
        let data = Arc::new(RwLock::new(ModelData {
            position: data.position,
            rotation: data.rotation,
            scale: data.scale,
            groups: data.groups.clone(),
        }));

        // This sender only errors when the receiver is dropped
        // which should only happen when the game is shutting down
        // so we ignore the error
        let _ = self.message_handle.send(UpdateMessage::NewModel {
            old_id: self.id,
            new_id,
            data: data.clone(),
        });

        ModelHandle {
            id: new_id,
            message_handle,
            data,
        }
    }
}

impl Drop for ModelHandle {
    fn drop(&mut self) {
        // This sender only errors when the receiver is dropped
        // which should only happen when the game is shutting down
        // so we ignore the error
        let _ = self
            .message_handle
            .send(UpdateMessage::ModelDropped(self.id));
    }
}

pub struct ModelRef {
    pub model: Arc<Model>,
    pub data: Arc<RwLock<ModelData>>,
}

impl ModelRef {
    pub fn new(
        model: Arc<Model>,
        message_handle: Sender<UpdateMessage>,
        mut data: ModelData,
    ) -> (u64, ModelRef, ModelHandle) {
        let id = ID.fetch_add(1, Ordering::Relaxed);
        let groups = (0..model.groups.len())
            .map(|_| ModelDataGroup::default())
            .collect();

        data.groups = groups;
        let data = Arc::new(RwLock::new(data));
        (
            id,
            ModelRef {
                model,
                data: data.clone(),
            },
            ModelHandle {
                id,
                data,
                message_handle,
            },
        )
    }
    pub fn with_new_data(&self, data: Arc<RwLock<ModelData>>) -> Self {
        ModelRef {
            model: self.model.clone(),
            data,
        }
    }
}
