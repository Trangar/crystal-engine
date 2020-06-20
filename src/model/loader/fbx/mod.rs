//! FBX.
mod data;
mod v7400;

use crate::error::ModelError;
use data::Scene;
use fbxcel_dom::{any::AnyDocument, fbxcel::low::FbxVersion, v7400::data::material::ShadingModel};
use std::path::Path;

/// Errors that can occur when loading an .fbx file
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The model has no polygon vertices
    #[error("No polygon vertices: {0:?}")]
    NoPolygonVertices(anyhow::Error),

    /// Could not triangulate the model
    #[error("Could not triangulate: {0:?}")]
    CouldNotTriangulate(anyhow::Error),

    /// The model mesh has no layers
    #[error("Mesh has no layer")]
    MeshHasNoLayer,

    /// The model mesh has no normals
    #[error("Mesh has no normals")]
    MeshHasNoNormals,

    /// The model mesh has no UV
    #[error("Mesh has no uv")]
    MeshHasNoUV,

    /// The model mesh has no materials
    #[error("Mesh has no materials")]
    MeshHasNoMaterials,

    /// The model's components should match in length, but they don't.
    #[error("Invalid mesh components, expected these values to be equal: {positions} positions, {normals} normals, {uv} UVs")]
    InvalidModelComponentCount {
        /// The amount of positions that were found
        positions: usize,
        /// The amount of normals that were found
        normals: usize,
        /// The amount of UVs that were found
        uv: usize,
    },

    /// The shading model of the model is incorrect
    #[error("Invalid shading model: {0:?}")]
    UnknownShadingModel(Result<ShadingModel, anyhow::Error>),

    /// Could not load geometry
    #[error("Could not load geometry: {0:?}")]
    CouldNotLoadGeometry(anyhow::Error),

    /// Could not load materials
    #[error("Could not load materials: {0:?}")]
    CouldNotLoadMaterials(Box<Error>),

    /// Could not load the geometry mesh
    #[error("Could not load geometry mesh: {0:?}")]
    CouldNotLoadGeometryMesh(Box<Error>),

    /// Could not wrap the UV axis
    #[error("Could not wrap UV axis: {0:?}")]
    CouldNotWrapUVAxis(anyhow::Error),

    /// The model is missing image data
    #[error("Missing image data")]
    MissingImageData,

    /// Could not load the video of the model
    #[error("Could not load video: {0:?}")]
    CouldNotLoadVideo(Box<Error>),

    /// Could not load the video file of the model
    #[error("Could not load video file: {0:?}")]
    CouldNotLoadVideoFile(anyhow::Error),

    /// The video file of the model has no content
    #[error("Video file has no content")]
    NoVideoContent,

    /// Could not interpret the image data as a TGA file
    #[error("Could not parse the texture as a TGA")]
    CouldNotLoadTgaImage(image::ImageError),

    /// Could not load the texture image
    #[error("Could not load the video image")]
    CouldNotLoadVideoImage(image::ImageError),

    /// Could not open the given file for reading
    #[error("Could not open {file:?} for reading: {inner:?}")]
    CouldNotOpenFile {
        /// The file that was trying to be loaded
        file: String,
        /// The inner exception
        inner: std::io::Error,
    },

    /// Could not parse the FBX document
    #[error("Could not parse document: {0:?}")]
    CouldNotParseDocument(fbxcel_dom::any::Error),

    /// The model is in an FBX format that can currently not be loaded.
    #[error(
        "Given model file is in an incorrect format, got {version:?}, expected one of {supported:?}"
    )]
    UnsupportedFormat {
        /// The version of the model
        version: FbxVersion,
        /// The versions that the engine can load
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
