use super::Model;
use cgmath::{Euler, Matrix4, Rad, Vector3, Zero};
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::Sender,
    Arc,
};

pub struct ModelData {
    pub(crate) id: u64,
    pub(crate) model: Arc<Model>,
    pub position: Vector3<f32>,
    pub rotation: Euler<Rad<f32>>,
    pub scale: f32,
}

impl ModelData {
    pub(self) fn new(model: Arc<Model>) -> Arc<RwLock<Self>> {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = ID.fetch_add(1, Ordering::Relaxed);
        Arc::new(RwLock::new(Self {
            id,
            model,
            position: Vector3::zero(),
            rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
            scale: 0.0,
        }))
    }
    pub(crate) fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale)
            * Matrix4::from_translation(self.position)
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
    ) -> (Self, Arc<RwLock<ModelData>>) {
        let data = ModelData::new(model);
        (
            Self {
                message_handle,
                data: data.clone(),
            },
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
        let data = self.data.clone();

        // This sender only errors when the receiver is dropped
        // which should only happen when the game is shutting down
        // so we ignore the error
        let _ = self
            .message_handle
            .send(ModelHandleMessage::NewClone(Arc::clone(&data)));

        Self {
            message_handle: self.message_handle.clone(),
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
            .send(ModelHandleMessage::Dropped(self.data.read().id));
    }
}

pub(crate) enum ModelHandleMessage {
    NewClone(Arc<RwLock<ModelData>>),
    Dropped(u64),
}
