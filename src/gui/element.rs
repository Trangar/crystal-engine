use parking_lot::RwLock;
use std::sync::Arc;
use vulkano::{
    device::Queue,
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sync::GpuFuture,
};

pub struct GuiElementRef {
    pub data: Arc<RwLock<GuiElementData>>,
    pub texture: Arc<ImmutableImage<R8G8B8A8Srgb>>,
    pub texture_future: Option<Box<dyn GpuFuture>>,
}

pub struct GuiElementData {
    pub dimensions: (i32, i32, u32, u32),
}
pub struct GuiElement {
    data: Arc<RwLock<GuiElementData>>,
}

impl GuiElement {
    pub(crate) fn new(
        queue: Arc<Queue>,
        dimensions: (i32, i32, u32, u32),
        image_data: (u32, u32, Vec<u8>),
    ) -> (GuiElementRef, GuiElement) {
        let (width, height, data) = image_data;
        let (texture, texture_future) = ImmutableImage::from_iter(
            data.into_iter(),
            Dimensions::Dim2d { width, height },
            R8G8B8A8Srgb,
            queue,
        )
        .unwrap();

        let data = Arc::new(RwLock::new(GuiElementData { dimensions }));

        (
            GuiElementRef {
                data: Arc::clone(&data),
                texture,
                texture_future: Some(texture_future.boxed()),
            },
            GuiElement { data },
        )
    }

    pub fn modify(&self, cb: impl FnOnce(&mut GuiElementData)) {
        let mut lock = self.data.write();
        cb(&mut *lock);
    }
}
