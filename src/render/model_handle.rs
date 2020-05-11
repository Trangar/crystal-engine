use super::Model;
use cgmath::{Euler, Matrix4, Rad, Vector3, Zero};
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::Sender,
    Arc,
};

/// Data of a model. This is behind an `Arc<RwLock<>>` so that the engine can keep a copy and check the latest values.
///
/// For an example on how to use this, see the example in the root of this module. This is the value passed in `ModelHandle::modify`.
pub struct ModelData {
    pub(crate) id: u64,
    pub(crate) model: Arc<Model>,
    /// The current position in the world that this model exists at.
    pub position: Vector3<f32>,
    /// The rotation of this model, in euler angles.
    pub rotation: Euler<Rad<f32>>,
    /// The scale of this model.
    pub scale: f32,
}

impl std::fmt::Debug for ModelData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ModelData")
            .field("position", &self.position)
            .field("rotation", &self.rotation)
            .field("scale", &self.scale)
            .finish()
    }
}

impl ModelData {
    pub(self) fn new(model: Arc<Model>) -> (u64, Arc<RwLock<Self>>) {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = ID.fetch_add(1, Ordering::Relaxed);
        (
            id,
            Arc::new(RwLock::new(Self {
                id,
                model,
                position: Vector3::zero(),
                rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
                scale: 1.0,
            })),
        )
    }
    pub(crate) fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale)
    }
}

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
    /// let handle: ModelHandle = game_state.create_triangle_model();
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
    /// let handle: ModelHandle = game_state.create_triangle_model();
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

pub(crate) enum ModelHandleMessage {
    NewClone(u64, Arc<RwLock<ModelData>>),
    Dropped(u64),
}
