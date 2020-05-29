//! Material.

use rgb::RGB;

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
    pub ambient: RGB<f32>,
    /// Diffuse.
    pub diffuse: RGB<f32>,
    /// Emissive.
    pub emissive: RGB<f32>,
}

impl Into<crate::render::Material> for Material {
    fn into(self) -> crate::render::Material {
        match self.data {
            ShadingData::Lambert(lambert) => crate::render::Material {
                ambient: lambert.ambient.into(),
                diffuse: lambert.diffuse.into(),
                specular: lambert.emissive.into(),
                shininess: 0.0,
            },
        }
    }
}
