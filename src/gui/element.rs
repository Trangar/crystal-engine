use crate::model::InternalUpdateMessage;
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::Sender,
    Arc,
};
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

impl GuiElementRef {
    pub(crate) fn with_new_data(&self, new_data: Arc<RwLock<GuiElementData>>) -> GuiElementRef {
        GuiElementRef {
            data: new_data,
            texture: self.texture.clone(),
            texture_future: None,
        }
    }
}

/// The data of a [GuiElement]. This can be used to manipulate an existing GuiElement.
pub struct GuiElementData {
    /// The dimensions of the [GuiElement].
    /// The format of this field is `(x, y, width, height)`.
    /// This means that the right edge would be `dimensions.0 + dimensions.2` and the bottom edge would be `dimensions.1 + dimensions.3`.
    pub dimensions: (i32, i32, u32, u32),
}

impl GuiElementData {
    pub(crate) fn from_old(old: &Self) -> Self {
        Self {
            dimensions: old.dimensions,
        }
    }
}

/// A reference to a GUI element on the screen.
///
/// This reference can be [Clone]d to create a second element on the screen with the exact same parameters that were used to create this.
///
/// This reference can be modified with the [modify](#method.modify) method.
pub struct GuiElement {
    id: u64,
    data: Arc<RwLock<GuiElementData>>,
    internal_update: Sender<InternalUpdateMessage>,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Clone for GuiElement {
    fn clone(&self) -> Self {
        let old_id = self.id;
        let new_id = ID.fetch_add(1, Ordering::Relaxed);
        let current_data = self.data.read();
        let data = Arc::new(RwLock::new(GuiElementData::from_old(&*current_data)));
        let _ = self
            .internal_update
            .send(InternalUpdateMessage::NewGuiElement(
                old_id,
                new_id,
                data.clone(),
            ));
        Self {
            id: new_id,
            data,
            internal_update: self.internal_update.clone(),
        }
    }
}

impl Drop for GuiElement {
    fn drop(&mut self) {
        let _ = self
            .internal_update
            .send(InternalUpdateMessage::GuiElementDropped(self.id));
    }
}

impl GuiElement {
    pub(crate) fn new(
        queue: Arc<Queue>,
        dimensions: (i32, i32, u32, u32),
        image_data: (u32, u32, Vec<u8>),
        internal_update: Sender<InternalUpdateMessage>,
    ) -> (u64, GuiElementRef, GuiElement) {
        let id = ID.fetch_add(1, Ordering::Relaxed);

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
            id,
            GuiElementRef {
                data: Arc::clone(&data),
                texture,
                texture_future: Some(texture_future.boxed()),
            },
            GuiElement {
                id,
                data,
                internal_update,
            },
        )
    }

    /// Modify the current GuiElement.
    pub fn modify(&self, cb: impl FnOnce(&mut GuiElementData)) {
        let mut lock = self.data.write();
        cb(&mut *lock);
    }
}
