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
use loader::ParsedModelPart;
use parking_lot::RwLock;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    format::R8G8B8A8Srgb,
    image::ImmutableImage,
    sync::GpuFuture,
};

// TODO: Make it so that developers can create their own models/vertices?
pub struct Model {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub groups: Vec<ModelGroup>,
    pub texture_future: RwLock<Vec<Box<dyn GpuFuture>>>,
}

pub struct ModelGroup {
    pub material: Option<Material>,
    pub texture: Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>,
    pub index: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
}

impl ModelGroup {
    pub fn from_tex(texture: Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>) -> Self {
        Self {
            material: None,
            texture,
            index: None,
        }
    }

    pub fn from_part(
        device: Arc<Device>,
        texture: &Option<Arc<ImmutableImage<R8G8B8A8Srgb>>>,
        part: ParsedModelPart,
    ) -> (Self, Option<Box<dyn GpuFuture>>) {
        let index = Some(
            CpuAccessibleBuffer::from_iter(
                device,
                BufferUsage::all(),
                false,
                part.index.iter().copied(),
            )
            .unwrap(),
        );

        (
            Self {
                material: None,
                texture: texture.clone(),
                index,
            },
            None,
        )
    }
}
