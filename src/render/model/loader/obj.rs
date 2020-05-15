use super::{CowIndex, CowVertex, ParsedModel};
use crate::render::Vertex;
use genmesh::EmitTriangles;

pub fn load(src: &str) -> ParsedModel {
    let mut obj = obj::Obj::load(std::path::Path::new(src)).expect("Could not load obj");
    obj.load_mtls().unwrap();
    let obj::ObjData {
        position,
        texture,
        normal,
        objects,
        // material_libs,
        ..
    } = obj.data;

    let vertices: Vec<_> = position
        .into_iter()
        .enumerate()
        .map(|(index, position)| Vertex {
            position_in: position,
            tex_coord_in: texture.get(index).cloned().unwrap_or([-1.0, -1.0]),
            normal_in: normal.get(index).cloned().unwrap_or([0.0, 0.0, 0.0]),
        })
        .collect();

    let mut indices: Vec<_> = Vec::with_capacity(objects.iter().map(|o| o.groups.len()).sum());
    for object in objects {
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

    (CowVertex::from(vertices), CowIndex::from(indices)).into()
}
