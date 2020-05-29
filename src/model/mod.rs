mod builder;
mod data;
mod handle;
mod loader;
mod render;

pub use self::{
    builder::ModelBuilder,
    data::ModelData,
    handle::{ModelHandle, ModelHandleMessage},
    loader::SourceOrShape,
    render::{fs, vs},
};

use crate::render::{Material, Vertex};
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
        let index = Some(
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                part.index.iter().copied(),
            )
            .unwrap(),
        );

        let vertex_buffer = part.vertices.map(|v| {
            CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, v.iter().copied())
                .unwrap()
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
            .unwrap();
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
