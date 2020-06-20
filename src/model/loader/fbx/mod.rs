//! FBX.
mod data;
mod v7400;

use crate::error::ModelError;
use data::Scene;
use fbxcel_dom::{any::AnyDocument, fbxcel::low::FbxVersion, v7400::data::material::ShadingModel};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No polygon vertices: {0:?}")]
    NoPolygonVertices(anyhow::Error),
    #[error("Could not triangulate: {0:?}")]
    CouldNotTriangulate(anyhow::Error),
    #[error("Mesh has no layer")]
    MeshHasNoLayer,
    #[error("Mesh has no normals")]
    MeshHasNoNormals,
    #[error("Mesh has no uv")]
    MeshHasNoUV,
    #[error("Mesh has no materials")]
    MeshHasNoMaterials,
    #[error("Invalid mesh components, expected these values to be equal: {positions} positions, {normals} normals, {uv} UVs")]
    InvalidModelComponentCount {
        positions: usize,
        normals: usize,
        uv: usize,
    },
    #[error("Invalid shading model: {0:?}")]
    UnknownShadingModel(Result<ShadingModel, anyhow::Error>),
    #[error("Could not load geometry: {0:?}")]
    CouldNotLoadGeometry(anyhow::Error),
    #[error("Could not load materials: {0:?}")]
    CouldNotLoadMaterials(Box<Error>),
    #[error("Could not load geometry mesh: {0:?}")]
    CouldNotLoadGeometryMesh(Box<Error>),
    #[error("Could not wrap UV axis: {0:?}")]
    CouldNotWrapUVAxis(anyhow::Error),
    #[error("Missing image data")]
    MissingImageData,
    #[error("Could not load video: {0:?}")]
    CouldNotLoadVideo(Box<Error>),
    #[error("Could not load video file: {0:?}")]
    CouldNotLoadVideoFile(anyhow::Error),
    #[error("Video file has no content")]
    NoVideoContent,
    #[error("Could not parse the texture as a TGA")]
    CouldNotLoadTgaImage(image::ImageError),
    #[error("Could not load the video image")]
    CouldNotLoadVideoImage(image::ImageError),
    #[error("Could not open {file:?} for reading: {inner:?}")]
    CouldNotOpenFile { file: String, inner: std::io::Error },
    #[error("Could not parse document: {0:?}")]
    CouldNotParseDocument(fbxcel_dom::any::Error),
    #[error(
        "Given model file is in an incorrect format, got {version:?}, expected one of {supported:?}"
    )]
    UnsupportedFormat {
        version: FbxVersion,
        supported: &'static [FbxVersion],
    },
}

/// Loads FBX data.
pub fn load(path: impl AsRef<Path>) -> Result<Scene, ModelError> {
    load_impl(path.as_ref()).map_err(ModelError::Fbx)
}

static SUPPORTED_VERSIONS: &[FbxVersion] = &[FbxVersion::V7_4];

/// Loads FBX data.
fn load_impl(path: &Path) -> Result<Scene, Error> {
    let file_name = path.to_str().unwrap_or("unknown");
    let file = std::io::BufReader::new(std::fs::File::open(path).map_err(|e| {
        Error::CouldNotOpenFile {
            file: file_name.to_string(),
            inner: e,
        }
    })?);
    match AnyDocument::from_seekable_reader(file).map_err(Error::CouldNotParseDocument)? {
        AnyDocument::V7400(_ver, doc) => v7400::from_doc(doc),
        x => Err(Error::UnsupportedFormat {
            version: x.fbx_version(),
            supported: SUPPORTED_VERSIONS,
        }),
    }
}
