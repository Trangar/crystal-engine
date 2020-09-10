use super::builder::TextRequest;
use crate::{error::GuiError, internal::UpdateMessage};
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicU32, AtomicU64, Ordering},
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

static NEXT_Z_INDEX: AtomicU32 = AtomicU32::new(1);

impl GuiElementRef {
    pub fn with_new_data(&self, new_data: Arc<RwLock<GuiElementData>>) -> GuiElementRef {
        GuiElementRef {
            data: new_data,
            texture: self.texture.clone(),
            texture_future: None,
        }
    }
}

/// The data of a [GuiElement]. This can be used to manipulate an existing GuiElement.
pub struct GuiElementData {
    /// The z-index of the element on the screen.
    /// Elements with a higher z-index are rendered on top of elements with a lower z-index.
    ///
    /// Elements will automatically be assigned a higher z-index than the last element created, so initially you can assume that newer elements are rendered on top.
    pub z_index: u32,

    /// The dimensions of the [GuiElement].
    /// The format of this field is `(x, y, width, height)`.
    /// This means that the right edge would be `dimensions.0 + dimensions.2` and the bottom edge would be `dimensions.1 + dimensions.3`.
    pub dimensions: (i32, i32, u32, u32),
}

/// A reference to a GUI element on the screen.
///
/// This reference can be [Clone]d to create a second element on the screen with the exact same parameters that were used to create this.
///
/// This reference can be modified with the [modify](#method.modify) method.
pub struct GuiElement {
    id: u64,
    data: Arc<RwLock<GuiElementData>>,
    internal_update: Sender<UpdateMessage>,
    canvas_config: Option<CanvasConfig>,
}

#[derive(Clone)]
pub(crate) struct CanvasConfig {
    pub background: [u8; 4],
    pub border: Option<(u16, [u8; 4])>,
    pub text: Option<TextRequest>,
}

static ID: AtomicU64 = AtomicU64::new(0);

impl Clone for GuiElement {
    fn clone(&self) -> Self {
        let old_id = self.id;
        let new_id = ID.fetch_add(1, Ordering::Relaxed);
        let data = self.data.read();
        let data = Arc::new(RwLock::new(GuiElementData {
            dimensions: data.dimensions,
            z_index: data.z_index,
        }));

        let _ = self.internal_update.send(UpdateMessage::NewGuiElement {
            old_id,
            new_id,
            data: data.clone(),
        });
        Self {
            id: new_id,
            data,
            internal_update: self.internal_update.clone(),
            canvas_config: self.canvas_config.clone(),
        }
    }
}

impl Drop for GuiElement {
    fn drop(&mut self) {
        let _ = self
            .internal_update
            .send(UpdateMessage::GuiElementDropped(self.id));
    }
}

impl GuiElement {
    pub(crate) fn new(
        queue: Arc<Queue>,
        dimensions: (i32, i32, u32, u32),
        image_data: (u32, u32, Vec<u8>),
        internal_update: Sender<UpdateMessage>,
        canvas_config: Option<CanvasConfig>,
    ) -> Result<(u64, GuiElementRef, GuiElement), GuiError> {
        let id = ID.fetch_add(1, Ordering::Relaxed);

        let (width, height, data) = image_data;
        let (texture, texture_future) = ImmutableImage::from_iter(
            data.into_iter(),
            Dimensions::Dim2d { width, height },
            R8G8B8A8Srgb,
            queue,
        )
        .map_err(|inner| GuiError::CouldNotCreateTexture { inner })?;

        let data = Arc::new(RwLock::new(GuiElementData {
            dimensions,
            z_index: NEXT_Z_INDEX.fetch_add(1, Ordering::Relaxed),
        }));

        Ok((
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
                canvas_config,
            },
        ))
    }

    /// Update the canvas. This will have the exact same settings as before, you can overwrite this by calling one of the helper methods on [GuiElementCanvasBuilder].
    ///
    /// This method will panic if the current GuiElement is created as a texture
    ///
    /// ```rust,no_run
    /// # use crystal_engine::{GameState, GuiElement, color};
    /// # let mut game_state: GameState = unsafe { std::mem::zeroed() };
    /// # let font = game_state.load_font("").unwrap();
    /// let mut gui_element: GuiElement = game_state.new_gui_element((0, 0, 100, 100))
    ///     .canvas()
    ///     .with_text(font.clone(), 32, "Hello world", color::WHITE)
    ///     .build()
    ///     .unwrap();
    /// gui_element.update_canvas(&mut game_state, |b| b.with_text_content("Hello you!")).unwrap();
    /// ```
    ///
    /// [GuiElementCanvasBuilder]: ../GuiElementCanvasBuilder.html
    pub fn update_canvas(
        &mut self,
        game_state: &mut crate::GameState,
        cb: impl FnOnce(super::GuiElementCanvasBuilder) -> super::GuiElementCanvasBuilder,
    ) -> Result<(), GuiError> {
        let canvas_config = self.canvas_config.clone().unwrap();
        let mut builder = super::GuiElementBuilder::new(game_state, self.data.read().dimensions)
            .canvas()
            .with_background_color(canvas_config.background);
        if let Some(border) = canvas_config.border {
            builder = builder.with_border(border.0, border.1);
        }
        if let Some(TextRequest {
            font,
            font_size,
            text,
            color,
        }) = canvas_config.text
        {
            builder = builder.with_text(font, font_size, text, color);
        }
        let builder = cb(builder);
        *self = builder.build()?;
        Ok(())
    }

    /// Modify the current GuiElement.
    pub fn modify(&self, cb: impl FnOnce(&mut GuiElementData)) {
        let mut lock = self.data.write();
        cb(&mut *lock);
    }
}
