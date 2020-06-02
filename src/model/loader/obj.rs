use super::{CowVertex, ParsedModel, ParsedModelPart};
use crate::model::{Material, Vertex};
use genmesh::EmitTriangles;
use obj::ObjMaterial;
use std::sync::Arc;

pub fn load(src: &str) -> ParsedModel {
    let mut obj = obj::Obj::load(std::path::Path::new(src)).expect("Could not load obj");
    obj.load_mtls().unwrap();
    let obj::ObjData {
        position,
        texture,
        normal,
        objects,
        material_libs,
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

    let mut result = ParsedModel::from(CowVertex::from(vertices));
    result
        .parts
        .reserve(objects.iter().map(|o| o.groups.len()).sum());

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

            let mut part: ParsedModelPart = index_group.into();
            let material = group.material.and_then(|m| match m {
                ObjMaterial::Mtl(mtl) => Some(mtl),
                ObjMaterial::Ref(name) => material_libs
                    .iter()
                    .flat_map(|m| &m.materials)
                    .find(|m| m.name == name)
                    .map(|m| Arc::clone(m)),
            });
            if let Some(material) = material {
                part.material = Some(Material {
                    ambient: material.ka.unwrap_or([1.0, 0.0, 0.0]),
                    diffuse: material.kd.unwrap_or([1.0, 0.0, 0.0]),
                    specular: material.ks.unwrap_or([1.0, 0.0, 0.0]),
                    shininess: material.km.unwrap_or(0.0),
                });
            }
            result.parts.push(part);
        }
    }

    result
}
