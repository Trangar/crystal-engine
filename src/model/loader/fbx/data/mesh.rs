//! Mesh.

use crate::model::loader::fbx::data::{GeometryMeshIndex, MaterialIndex};

/// Mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name.
    pub name: Option<String>,
    /// Geometry mesh index.
    pub geometry_mesh_index: GeometryMeshIndex,
    /// Materials.
    pub materials: Vec<MaterialIndex>,
}
