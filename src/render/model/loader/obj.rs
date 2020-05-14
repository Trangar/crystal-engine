use super::{CowIndex, CowVertex};
use crate::render::Vertex;
use genmesh::EmitTriangles;

pub fn load(src: &str) -> (CowVertex, CowIndex) {
    let mut obj = obj::Obj::load(std::path::Path::new(src)).expect("Could not load obj");
    obj.load_mtls().unwrap();
    let obj = obj.data;

    let mut vertices = Vec::with_capacity(obj.position.len());
    for (index, position) in obj.position.into_iter().enumerate() {
        vertices.push(Vertex {
            position_in: position,
            tex_coord_in: obj.texture.get(index).cloned().unwrap_or([-1.0, -1.0]),
            normal_in: obj.normal.get(index).cloned().unwrap_or([0.0, 0.0, 0.0]),
        });
    }

    let mut indices: Vec<_> = Vec::with_capacity(obj.objects.iter().map(|o| o.groups.len()).sum());
    for object in obj.objects {
        for group in object.groups {
            let mut index_group = Vec::new();
            for poly in group.polys {
                poly.into_genmesh().emit_triangles(|triangle| {
                    index_group.push(triangle.x.0 as u32);
                    index_group.push(triangle.y.0 as u32);
                    index_group.push(triangle.z.0 as u32);
                });
            }
            indices.push(index_group.into());
        }
    }

    (vertices.into(), indices.into())
}
