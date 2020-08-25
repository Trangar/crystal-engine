//! Geometry.

use vek::{Vec2, Vec3};

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeometryMesh {
    /// Name.
    pub name: Option<String>,
    /// Positions.
    pub positions: Vec<Vec3<f32>>,
    /// Normals.
    pub normals: Vec<Vec3<f32>>,
    /// UV.
    pub uv: Vec<Vec2<f32>>,
    /// Indices per materials.
    pub indices_per_material: Vec<Vec<u32>>,
}
