//! FBX.
mod data;
mod v7400;

use data::Scene;
use fbxcel_dom::{any::AnyDocument, fbxcel::low::FbxVersion};
use std::{convert::Infallible, path::Path};

/// Loads FBX data.
pub fn load(path: impl AsRef<Path>) -> Result<Scene, Infallible> {
    load_impl(path.as_ref())
}

static SUPPORTED_VERSIONS: &[FbxVersion] = &[FbxVersion::V7_4];

/// Loads FBX data.
fn load_impl(path: &Path) -> Result<Scene, Infallible> {
    let file = std::io::BufReader::new(std::fs::File::open(path).expect("Could not open file"));
    match AnyDocument::from_seekable_reader(file).expect("Could not read FBX") {
        AnyDocument::V7400(_ver, doc) => v7400::from_doc(doc),
        x => panic!(
            "Version {:?} not supported, one of the following versions was expected: {:?}",
            x.fbx_version(),
            SUPPORTED_VERSIONS
        ),
    }
}
