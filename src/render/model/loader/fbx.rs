use super::{CowIndex, CowVertex};
use fbxcel_dom::{
    any::AnyDocument,
    v7400::object::{geometry::MeshHandle, model::TypedModelHandle, TypedObjectHandle},
};

pub fn load(src: &str) -> (CowVertex, CowIndex) {
    let file = std::fs::File::open(src).expect("Failed to open file");
    // You can also use raw `file`, but do buffering for better efficiency.
    let reader = std::io::BufReader::new(file);

    // Use `from_seekable_reader` for readers implementing `std::io::Seek`.
    // To use readers without `std::io::Seek` implementation, use `from_reader`
    // instead.
    match AnyDocument::from_seekable_reader(reader).expect("Failed to load document") {
        AnyDocument::V7400(_fbx_ver, doc) => {
            for object in doc.objects().filter_map(|o| {
                if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = o.get_typed() {
                    Some(mesh)
                } else {
                    None
                }
            }) {
                let geometry: MeshHandle = object.geometry().unwrap();
            }
            // You got a document. You can do what you want.
            unimplemented!()
        }
        // `AnyDocument` is nonexhaustive.
        // You should handle unknown document versions case.
        _ => panic!("Got FBX document of unsupported version"),
    }
}
