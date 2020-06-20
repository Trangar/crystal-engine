//! Scene.

use crate::model::{
    loader::{
        fbx::data::{GeometryMesh, Material, Mesh, Texture},
        ParsedModel, ParsedModelPart, ParsedTexture,
    },
    Vertex,
};

/// Scene.
#[derive(Default, Debug, Clone)]
pub struct Scene {
    /// Scene name.
    name: Option<String>,
    /// Geometry mesh.
    geometry_meshes: Vec<GeometryMesh>,
    /// Materials.
    materials: Vec<Material>,
    /// Meshes.
    meshes: Vec<Mesh>,
    /// Textures.
    textures: Vec<Texture>,
}

impl Scene {
    /// Add a geometry mesh.
    pub(crate) fn add_geometry_mesh(&mut self, mesh: GeometryMesh) -> GeometryMeshIndex {
        let index = GeometryMeshIndex::new(self.meshes.len());
        self.geometry_meshes.push(mesh);
        index
    }

    /// Returns a reference to the geometry mesh.
    pub fn geometry_mesh(&self, i: GeometryMeshIndex) -> Option<&GeometryMesh> {
        self.geometry_meshes.get(i.to_usize())
    }

    /// Add a material.
    pub(crate) fn add_material(&mut self, material: Material) -> MaterialIndex {
        let index = MaterialIndex::new(self.materials.len());
        self.materials.push(material);
        index
    }

    /// Returns a reference to the material.
    pub fn material(&self, i: MaterialIndex) -> Option<&Material> {
        self.materials.get(i.to_usize())
    }

    /// Add a mesh.
    pub(crate) fn add_mesh(&mut self, mesh: Mesh) -> MeshIndex {
        let index = MeshIndex::new(self.meshes.len());
        self.meshes.push(mesh);
        index
    }

    /// Add a texture.
    pub(crate) fn add_texture(&mut self, texture: Texture) -> TextureIndex {
        let index = TextureIndex::new(self.textures.len());
        self.textures.push(texture);
        index
    }

    /// Returns a reference to the texture.
    pub fn texture(&self, i: TextureIndex) -> Option<&Texture> {
        self.textures.get(i.to_usize())
    }
}

/// Mesh index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshIndex(u32);

impl MeshIndex {
    /// Creates a new index.
    ///
    /// # Panics
    ///
    /// Panics if the given index is larger than `std::u32::MAX`.
    pub(crate) fn new(i: usize) -> Self {
        assert!(i <= std::u32::MAX as usize);
        Self(i as u32)
    }
}

macro_rules! define_index_type {
    ($(
        $(#[$meta:meta])*
        $ty:ident;
    )*) => {
        $(
            define_index_type! {
                @single
                $(#[$meta])*
                $ty;
            }
        )*
    };
    (
        @single
        $(#[$meta:meta])*
        $ty:ident;
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ty(u32);

        impl $ty {
            /// Creates a new index.
            ///
            /// # Panics
            ///
            /// Panics if the given index is larger than `std::u32::MAX`.
            pub(crate) fn new(i: usize) -> Self {
                assert!(i <= std::u32::MAX as usize);
                Self(i as u32)
            }

            /// Retuns `usize` value.
            pub fn to_usize(self) -> usize {
                self.0 as usize
            }
        }
    };
}

define_index_type! {
    /// Geometry mesh index.
    GeometryMeshIndex;
    /// Material index.
    MaterialIndex;
    /// Texture index.
    TextureIndex;
}

impl Into<ParsedModel> for Scene {
    fn into(self) -> ParsedModel {
        let mut parts = Vec::new();

        for mesh in &self.meshes {
            // we assume the mesh exists, else the parser should have returned an error
            let geometry = self.geometry_mesh(mesh.geometry_mesh_index).unwrap();
            for (i, indices) in geometry.indices_per_material.iter().enumerate() {
                let material = self.material(mesh.materials[i]);

                let texture: Option<ParsedTexture> = material
                    .and_then(|m| m.diffuse_texture)
                    .and_then(|i| self.texture(i))
                    .map(|texture| texture.clone().into());

                let vertices = geometry
                    .positions
                    .iter()
                    .zip(geometry.normals.iter())
                    .zip(geometry.uv.iter())
                    .map(|((position, normal), uv)| Vertex {
                        position_in: position.clone().into(),
                        normal_in: normal.clone().into(),
                        tex_coord_in: uv.clone().into(),
                    })
                    .collect();

                parts.push(ParsedModelPart {
                    index: indices.clone().into(),
                    material: material.cloned().map(Into::into),
                    vertices: Some(vertices),
                    texture,
                });
            }
        }

        ParsedModel {
            parts,
            vertices: None,
        }
    }
}
