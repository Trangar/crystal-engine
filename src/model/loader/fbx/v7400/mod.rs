//! FBX v7400 support.

use self::triangulator::triangulator;
use super::{
    data::{
        GeometryMesh, GeometryMeshIndex, LambertData, Material, MaterialIndex, Mesh, MeshIndex,
        Scene, ShadingData, Texture, TextureIndex, WrapMode,
    },
    Error,
};
use fbxcel_dom::v7400::{
    data::{
        material::ShadingModel, mesh::layer::TypedLayerElementHandle,
        texture::WrapMode as RawWrapMode,
    },
    object::{self, model::TypedModelHandle, ObjectId, TypedObjectHandle},
    Document,
};
use std::{collections::HashMap, path::Path};
use vek::{Vec2, Vec3};

mod triangulator;

type Result<T = ()> = std::result::Result<T, Error>;

/// Loads the data from the document.
pub fn from_doc(doc: Box<Document>) -> Result<Scene> {
    Loader::new(&doc).load()
}

/// FBX data loader.
pub struct Loader<'a> {
    /// Document.
    doc: &'a Document,
    /// Scene.
    scene: Scene,
    /// Geometry mesh indices.
    geometry_mesh_indices: HashMap<ObjectId, GeometryMeshIndex>,
    /// Material indices.
    material_indices: HashMap<ObjectId, MaterialIndex>,
    /// Mesh indices.
    mesh_indices: HashMap<ObjectId, MeshIndex>,
    /// Texture indices.
    texture_indices: HashMap<ObjectId, TextureIndex>,
}

impl<'a> Loader<'a> {
    /// Creates a new `Loader`.
    fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            scene: Default::default(),
            geometry_mesh_indices: Default::default(),
            material_indices: Default::default(),
            mesh_indices: Default::default(),
            texture_indices: Default::default(),
        }
    }

    /// Loads the document.
    fn load(mut self) -> Result<Scene> {
        for obj in self.doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                self.load_mesh(mesh)?;
            }
        }

        Ok(self.scene)
    }

    /// Loads the geometry.
    fn load_geometry_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle<'a>,
        num_materials: usize,
    ) -> Result<GeometryMeshIndex> {
        if let Some(index) = self.geometry_mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        let polygon_vertices = mesh_obj
            .polygon_vertices()
            .map_err(Error::NoPolygonVertices)?;
        let triangle_pvi_indices = polygon_vertices
            .triangulate_each(triangulator)
            .map_err(Error::CouldNotTriangulate)?;

        let positions = triangle_pvi_indices
            .iter_control_point_indices()
            .filter_map(|cpi| cpi)
            .filter_map(|cpi| polygon_vertices.control_point(cpi).map(Vec3::<f64>::from))
            .map(|p| p.as_())
            .collect::<Vec<_>>();

        let layer = mesh_obj.layers().next().ok_or(Error::MeshHasNoLayer)?;

        let normals: Vec<Vec3<f32>> = {
            let normals = layer
                .layer_element_entries()
                .filter_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Normal(handle)) => Some(handle),
                    _ => None,
                })
                .next()
                .and_then(|n| n.normals().ok())
                .ok_or(Error::MeshHasNoNormals)?;
            triangle_pvi_indices
                .triangle_vertex_indices()
                .filter_map(|tri_vi| {
                    normals
                        .normal(&triangle_pvi_indices, tri_vi)
                        .map(Vec3::<f64>::from)
                        .ok()
                })
                .map(|v| v.as_::<f32>())
                .collect::<Vec<_>>()
        };
        let uv: Vec<Vec2<f32>> = {
            let uv = layer
                .layer_element_entries()
                .filter_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Uv(handle)) => Some(handle),
                    _ => None,
                })
                .next()
                .and_then(|uv| uv.uv().ok())
                .ok_or(Error::MeshHasNoUV)?;
            triangle_pvi_indices
                .triangle_vertex_indices()
                .filter_map(|tri_vi| uv.uv(&triangle_pvi_indices, tri_vi).ok())
                .map(|p| Vec2::<f64>::from(p).as_::<f32>())
                .collect::<Vec<_>>()
        };

        let indices_per_material = {
            let mut indices_per_material = vec![Vec::new(); num_materials];
            let materials = layer
                .layer_element_entries()
                .filter_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Material(handle)) => Some(handle),
                    _ => None,
                })
                .next()
                .and_then(|l| l.materials().ok())
                .ok_or(Error::MeshHasNoMaterials)?;
            for tri_vi in triangle_pvi_indices.triangle_vertex_indices() {
                let target = materials
                    .material_index(&triangle_pvi_indices, tri_vi)
                    .ok()
                    .map(|l| l.to_u32())
                    .and_then(|i| indices_per_material.get_mut(i as usize));
                if let Some(target) = target {
                    target.push(tri_vi.to_usize() as u32);
                }
            }
            indices_per_material
        };

        if positions.len() != normals.len() || positions.len() != uv.len() {
            return Err(Error::InvalidModelComponentCount {
                positions: positions.len(),
                normals: normals.len(),
                uv: uv.len(),
            });
        }

        let mesh = GeometryMesh {
            name: mesh_obj.name().map(Into::into),
            positions,
            normals,
            uv,
            indices_per_material,
        };

        Ok(self.scene.add_geometry_mesh(mesh))
    }

    /// Loads the material.
    fn load_material(
        &mut self,
        material_obj: object::material::MaterialHandle<'a>,
    ) -> Result<MaterialIndex> {
        if let Some(index) = self.material_indices.get(&material_obj.object_id()) {
            return Ok(*index);
        }

        let diffuse_texture = material_obj
            .transparent_texture()
            .map(|v| (true, v))
            .or_else(|| material_obj.diffuse_texture().map(|v| (false, v)))
            .and_then(|(transparent, texture_obj)| {
                self.load_texture(texture_obj, transparent).ok()
            });

        let properties = material_obj.properties();
        let shading_data = match properties.shading_model_or_default() {
            Ok(ShadingModel::Lambert) | Ok(ShadingModel::Phong) => {
                let ambient_color = properties.ambient_color_or_default().unwrap_or_default();
                let ambient_factor = properties.ambient_factor_or_default().unwrap_or_default();
                let ambient = ambient_color * ambient_factor;
                let diffuse_color = properties.diffuse_color_or_default().unwrap_or_default();
                let diffuse_factor = properties.diffuse_factor_or_default().unwrap_or_default();
                let diffuse = diffuse_color * diffuse_factor;
                let emissive_color = properties.emissive_color_or_default().unwrap_or_default();
                let emissive_factor = properties.emissive_factor_or_default().unwrap_or_default();
                let emissive = emissive_color * emissive_factor;
                ShadingData::Lambert(LambertData {
                    ambient: [ambient.r as f32, ambient.g as f32, ambient.b as f32],
                    diffuse: [diffuse.r as f32, diffuse.g as f32, diffuse.b as f32],
                    emissive: [emissive.r as f32, emissive.g as f32, emissive.b as f32],
                })
            }
            v => return Err(Error::UnknownShadingModel(v)),
        };

        let material = Material {
            name: material_obj.name().map(Into::into),
            diffuse_texture,
            data: shading_data,
        };

        Ok(self.scene.add_material(material))
    }

    /// Loads the mesh.
    fn load_mesh(&mut self, mesh_obj: object::model::MeshHandle<'a>) -> Result<MeshIndex> {
        if let Some(index) = self.mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        let geometry_obj = mesh_obj.geometry().map_err(Error::CouldNotLoadGeometry)?;

        let materials = mesh_obj
            .materials()
            .map(|material_obj| self.load_material(material_obj))
            .collect::<Result<Vec<_>>>()
            .map_err(|e| Error::CouldNotLoadMaterials(Box::new(e)))?;

        let geometry_index = self
            .load_geometry_mesh(geometry_obj, materials.len())
            .map_err(|e| Error::CouldNotLoadGeometryMesh(Box::new(e)))?;

        let mesh = Mesh {
            name: mesh_obj.name().map(Into::into),
            geometry_mesh_index: geometry_index,
            materials,
        };

        Ok(self.scene.add_mesh(mesh))
    }

    /// Loads the texture.
    fn load_texture(
        &mut self,
        texture_obj: object::texture::TextureHandle<'a>,
        transparent: bool,
    ) -> Result<TextureIndex> {
        if let Some(index) = self.texture_indices.get(&texture_obj.object_id()) {
            return Ok(*index);
        }

        let properties = texture_obj.properties();
        let wrap_mode_u = {
            let val = properties
                .wrap_mode_u_or_default()
                .map_err(Error::CouldNotWrapUVAxis)?;
            match val {
                RawWrapMode::Repeat => WrapMode::Repeat,
                RawWrapMode::Clamp => WrapMode::ClampToEdge,
            }
        };
        let wrap_mode_v = {
            let val = properties
                .wrap_mode_v_or_default()
                .map_err(Error::CouldNotWrapUVAxis)?;
            match val {
                RawWrapMode::Repeat => WrapMode::Repeat,
                RawWrapMode::Clamp => WrapMode::ClampToEdge,
            }
        };
        let video_clip_obj = texture_obj.video_clip().ok_or(Error::MissingImageData)?;
        let image = self
            .load_video_clip(video_clip_obj)
            .map_err(|e| Error::CouldNotLoadVideo(Box::new(e)))?;

        let texture = Texture {
            name: texture_obj.name().map(Into::into),
            image,
            transparent,
            wrap_mode_u,
            wrap_mode_v,
        };

        Ok(self.scene.add_texture(texture))
    }

    /// Loads the texture image.
    fn load_video_clip(
        &mut self,
        video_clip_obj: object::video::ClipHandle<'a>,
    ) -> Result<image::DynamicImage> {
        let relative_filename = video_clip_obj
            .relative_filename()
            .map_err(Error::CouldNotLoadVideoFile)?;
        let file_ext = Path::new(&relative_filename)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_ascii_lowercase);
        let content = video_clip_obj.content().ok_or(Error::NoVideoContent)?;
        let image = match file_ext.as_ref().map(AsRef::as_ref) {
            Some("tga") => image::load_from_memory_with_format(content, image::ImageFormat::Tga)
                .map_err(Error::CouldNotLoadTgaImage)?,
            _ => image::load_from_memory(content).map_err(Error::CouldNotLoadVideoImage)?,
        };

        Ok(image)
    }
}
