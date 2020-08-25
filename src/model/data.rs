use vek::{Mat4, Vec3};

/// Data of a model. This is behind an `Arc<RwLock<>>` so that the engine can keep a copy and check the latest values.
///
/// For an example on how to use this, see the example in the root of this module. This is the value passed in `ModelHandle::modify`.
#[derive(Debug)]
pub struct ModelData {
    /// The current position in the world that this model exists at.
    pub position: Vec3<f32>,

    /// The rotation of this model, in euler angles.
    pub rotation: Vec3<f32>,

    /// The scale of this model.
    pub scale: f32,

    /// Contains the data of the groups in the model.
    /// If your 3d model has multiple parts, you can move them individually with this property.
    pub groups: Vec<ModelDataGroup>,
}

impl Default for ModelData {
    fn default() -> ModelData {
        Self {
            position: Vec3::zero(),
            rotation: Vec3::zero(),
            scale: 1.0,
            groups: Vec::new(),
        }
    }
}

impl ModelData {
    pub(crate) fn matrix(&self) -> Mat4<f32> {
        Mat4::<f32>::translation_3d(self.position)
            * Mat4::rotation_3d(1.0, self.rotation)
            * Mat4::scaling_3d::<f32>(self.scale)
    }
}

#[derive(Debug, Clone)]
pub struct ModelDataGroup {
    pub matrix: Mat4<f32>,
}

impl Default for ModelDataGroup {
    fn default() -> Self {
        Self {
            matrix: Mat4::identity(),
        }
    }
}
