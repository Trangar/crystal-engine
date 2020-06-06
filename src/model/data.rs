use cgmath::{Euler, Matrix4, Rad, SquareMatrix, Vector3, Zero};

/// Data of a model. This is behind an `Arc<RwLock<>>` so that the engine can keep a copy and check the latest values.
///
/// For an example on how to use this, see the example in the root of this module. This is the value passed in `ModelHandle::modify`.
#[derive(Debug)]
pub struct ModelData {
    /// The current position in the world that this model exists at.
    pub position: Vector3<f32>,

    /// The rotation of this model, in euler angles.
    pub rotation: Euler<Rad<f32>>,

    /// The scale of this model.
    pub scale: f32,

    /// Contains the data of the groups in the model.
    /// If your 3d model has multiple parts, you can move them individually with this property.
    pub groups: Vec<ModelDataGroup>,
}

impl Default for ModelData {
    fn default() -> ModelData {
        Self {
            position: Vector3::zero(),
            rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
            scale: 1.0,
            groups: Vec::new(),
        }
    }
}

impl ModelData {
    pub(crate) fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale)
    }
}

#[derive(Debug, Clone)]
pub struct ModelDataGroup {
    pub matrix: Matrix4<f32>,
}

impl Default for ModelDataGroup {
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }
}
