//! Geometry.

use crate::math::{Vector2, Vector3};

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeometryMesh {
    /// Name.
    pub name: Option<String>,
    /// Positions.
    pub positions: Vec<Vector3>,
    /// Normals.
    pub normals: Vec<Vector3>,
    /// UV.
    pub uv: Vec<Vector2>,
    /// Indices per materials.
    pub indices_per_material: Vec<Vec<u32>>,
}
