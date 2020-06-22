mod builder;
mod data;
mod handle;
pub mod loader;
mod pipeline;

pub use self::{
    builder::ModelBuilder,
    data::{ModelData, ModelDataGroup},
    handle::{ModelHandle, ModelRef},
    loader::SourceOrShape,
    pipeline::{vs, Pipeline},
};

#[cfg(feature = "format-fbx")]
pub use self::loader::fbx::Error as FbxError;

#[cfg(feature = "format-obj")]
pub use self::loader::obj::Error as ObjError;

use loader::{ParsedModelPart, ParsedTexture};
use parking_lot::RwLock;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::{Device, Queue},
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sync::GpuFuture,
};

// TODO: Make it so that developers can create their own models/vertices?
pub struct Model {
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub groups: Vec<ModelGroup>,
    pub texture_future: RwLock<Vec<Box<dyn GpuFuture>>>,
}

pub struct ModelGroup {
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub material: Option<Material>,
    pub texture: Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>,
    pub index: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
}

impl ModelGroup {
    pub fn from_tex(texture: Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>) -> Self {
        Self {
            vertex_buffer: None,
            material: None,
            texture,
            index: None,
        }
    }

    pub fn from_part(
        device: Arc<Device>,
        queue: Arc<Queue>,
        texture: &Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>,
        part: ParsedModelPart,
    ) -> (Self, Option<Box<dyn GpuFuture>>) {
        let index = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            part.index.iter().copied(),
        )
        .ok();

        let vertex_buffer = part.vertices.map(|v| {
            CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, v.iter().copied())
                .unwrap() // We assume that device and v are valid, so this should never fail
        });

        let (texture, future) = if let Some(texture_to_load) = part.texture {
            let ParsedTexture {
                width,
                height,
                rgba_data,
            } = texture_to_load;
            let (tex, fut) = ImmutableImage::from_iter(
                rgba_data.into_iter(),
                Dimensions::Dim2d { width, height },
                R8G8B8A8Srgb,
                queue,
            )
            .unwrap(); // We assume that queue, rgba_data and width/height are valid, so this should never fail
            (Some(tex), Some(Box::new(fut) as Box<dyn GpuFuture>))
        } else {
            (texture.clone(), None)
        };

        (
            Self {
                vertex_buffer,
                material: None,
                texture,
                index,
            },
            future,
        )
    }
}

#[derive(Default, Copy, Clone)]
/// A single vertex.
pub struct Vertex {
    /// The position of this vertex
    pub position: [f32; 3],

    /// The normal of this vertex
    pub normal: [f32; 3],

    /// The texture coordinate of this vertex
    pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, normal, tex_coord);

#[derive(Copy, Clone, Debug)]
/// The material of a model part. See the lights module for more information
pub struct Material {
    /// The ambient color multiplier of this material
    pub ambient: [f32; 3],
    /// The diffuse color multiplier of this material
    pub diffuse: [f32; 3],
    /// The specular color multiplier of this material
    pub specular: [f32; 3],
    /// The shininess multiplier of this material
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambient: [1.0, 1.0, 1.0],
            diffuse: [1.0, 1.0, 1.0],
            specular: [1.0, 1.0, 1.0],
            shininess: 1.0,
        }
    }
}
