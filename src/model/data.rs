use super::Model;
use crate::math::{prelude::*, rad, Euler, Matrix4, Vector3};
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// Data of a model. This is behind an `Arc<RwLock<>>` so that the engine can keep a copy and check the latest values.
///
/// For an example on how to use this, see the example in the root of this module. This is the value passed in `ModelHandle::modify`.
pub struct ModelData {
    pub(crate) id: u64,
    pub(crate) model: Arc<Model>,

    /// The current position in the world that this model exists at.
    pub position: Vector3,

    /// The rotation of this model, in euler angles.
    pub rotation: Euler,

    /// The scale of this model.
    pub scale: f32,

    /// Contains the data of the groups in the model.
    /// If your 3d model has multiple parts, you can move them individually with this property.
    pub groups: Vec<ModelDataGroup>,
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
    pub(crate) fn new(model: Arc<Model>) -> (u64, Arc<RwLock<Self>>) {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = ID.fetch_add(1, Ordering::Relaxed);
        let groups = (0..model.groups.len())
            .map(|_| ModelDataGroup::default())
            .collect();

        (
            id,
            Arc::new(RwLock::new(Self {
                id,
                model,
                position: Vector3::zero(),
                rotation: Euler::new(rad(0.0), rad(0.0), rad(0.0)),
                scale: 1.0,
                groups,
            })),
        )
    }
    pub(crate) fn matrix(&self) -> Matrix4 {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale)
    }
}

pub struct ModelDataGroup {
    pub matrix: Matrix4,
}

impl Default for ModelDataGroup {
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }
}
