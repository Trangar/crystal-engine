//! Material.

use crate::model::loader::fbx::data::TextureIndex;

/// Material.
#[derive(Debug, Clone)]
pub struct Material {
    /// Name.
    pub name: Option<String>,
    /// Texture index.
    pub diffuse_texture: Option<TextureIndex>,
    /// Shading parameters.
    pub data: ShadingData,
}

/// Shading data.
#[derive(Debug, Clone, Copy)]
pub enum ShadingData {
    /// Lambert material.
    Lambert(LambertData),
}

/// Lambert data.
#[derive(Debug, Clone, Copy)]
pub struct LambertData {
    /// Ambient.
    pub ambient: [f32; 3],
    /// Diffuse.
    pub diffuse: [f32; 3],
    /// Emissive.
    pub emissive: [f32; 3],
}

impl Into<crate::model::Material> for Material {
    fn into(self) -> crate::model::Material {
        match self.data {
            ShadingData::Lambert(lambert) => crate::model::Material {
                ambient: lambert.ambient,
                diffuse: lambert.diffuse,
                specular: lambert.emissive,
                shininess: 0.0,
            },
        }
    }
}
