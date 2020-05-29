//! Geometry.

use cgmath::{Point2, Point3, Vector3};

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeometryMesh {
    /// Name.
    pub name: Option<String>,
    /// Positions.
    pub positions: Vec<Point3<f32>>,
    /// Normals.
    pub normals: Vec<Vector3<f32>>,
    /// UV.
    pub uv: Vec<Point2<f32>>,
    /// Indices per materials.
    pub indices_per_material: Vec<Vec<u32>>,
}
