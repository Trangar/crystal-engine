use super::GuiElement;
use crate::{error::GuiError, Font, GameState};
use image::Pixel;

/// A struct that is used to create a [GuiElement]. It is constructed by calling `GameState::add_new_element()`
///
/// This builder can either load a texture by calling [with_texture], or you can create a custom image by calling [with_canvas].
///
/// [with_texture]: #method.with_texture
/// [with_canvas]: #method.with_canvas
pub struct GuiElementBuilder<'a> {
    game_state: &'a mut GameState,
    dimensions: (i32, i32, u32, u32),
}

impl<'a> GuiElementBuilder<'a> {
    pub(crate) fn new(game_state: &'a mut GameState, dimensions: (i32, i32, u32, u32)) -> Self {
        Self {
            game_state,
            dimensions,
        }
    }

    /// Create a gui element with a texture
    pub fn with_texture<'b>(self, texture_path: &'b str) -> GuiElementTextureBuilder<'a, 'b> {
        GuiElementTextureBuilder {
            game_state: self.game_state,
            dimensions: self.dimensions,
            texture_path,
        }
    }

    /// Create a gui element with a custom color. The returned [GuiElementCanvasBuilder] can be further changed to include text and borders.
    pub fn with_canvas(self, background_color: [u8; 4]) -> GuiElementCanvasBuilder<'a> {
        GuiElementCanvasBuilder {
            game_state: self.game_state,
            dimensions: self.dimensions,
            color: background_color,
            text: None,
            border: None,
        }
    }
}

/// A struct that is used to create a [GuiElement] with a texture. This is created by calling `GameState::create_gui_element().texture("..")`. Currently nothing can be manipulated in this struct.
pub struct GuiElementTextureBuilder<'a, 'b> {
    game_state: &'a mut GameState,
    dimensions: (i32, i32, u32, u32),
    texture_path: &'b str,
}
impl<'a, 'b> GuiElementTextureBuilder<'a, 'b> {
    /// Finish building the element and return it.
    /// The returned [GuiElement] has to be stored somewhere, as it will be removed from the engine when dropped.
    /// Starting next frame, the returned GuiElement will be rendered on the screen.
    pub fn build(self) -> Result<GuiElement, GuiError> {
        let queue = self.game_state.queue.clone();
        let image = image::open(self.texture_path)
            .map_err(|e| GuiError::CouldNotLoadTexture {
                path: self.texture_path.to_owned(),
                inner: e,
            })?
            .to_rgba();

        let (id, element_ref, element) = GuiElement::new(
            queue,
            self.dimensions,
            (image.width(), image.height(), image.into_raw()),
            self.game_state.internal_update_sender.clone(),
            None,
        )?;
        self.game_state.gui_elements.insert(id, element_ref);

        Ok(element)
    }
}
/// A struct that is used to render a custom texture for a [GuiElement]. This can be further customized by e.g. `.with_text` and `with_border`.
/// Finalize this GuiElement by calling `.build()`.
pub struct GuiElementCanvasBuilder<'a> {
    game_state: &'a mut GameState,
    dimensions: (i32, i32, u32, u32),
    color: [u8; 4],
    text: Option<TextRequest>,
    border: Option<(u16, [u8; 4])>,
}

#[derive(Clone)]
pub(crate) struct TextRequest {
    pub font: Font,
    pub font_size: u16,
    pub text: String,
    pub color: [u8; 4],
}

impl<'a> GuiElementCanvasBuilder<'a> {
    /// Adds a border to the [GuiElement].
    /// This will be subtracted from the size of the element,
    /// e.g. if you have an element of 100 pixels wide with a border of 10 pixels the resulting outer width will still be 100 pixels,
    /// while the inner width will be `100 - (left_border + right_border) = 100 - (10 + 10) = 80` pixels.
    pub fn with_border(mut self, border_width: u16, border_color: [u8; 4]) -> Self {
        self.border = Some((border_width, border_color));
        self
    }
    /// Add a text to the GUI element. This text will be rendered in the center of the element, and does not respect newlines.
    ///
    /// An instance of [Font](rusttype::Font) can be obtained by calling `GameState::load_font`.
    pub fn with_text(
        mut self,
        font: Font,
        font_size: u16,
        text: impl std::fmt::Display,
        color: [u8; 4],
    ) -> Self {
        self.text = Some(TextRequest {
            font,
            font_size,
            text: text.to_string(),
            color,
        });
        self
    }

    /// Update the text of an element. This has to be called *after* `with_text` is called. This is mostly useful when calling `GuiElement::rebuild_canvas`.
    pub fn with_text_content(mut self, text: impl std::fmt::Display) -> Self {
        self.text.as_mut().unwrap().text = text.to_string();
        self
    }

    /// Finish building the element and return it.
    /// The returned [GuiElement] has to be stored somewhere, as it will be removed from the engine when dropped.
    /// Starting next frame, the returned GuiElement will be rendered on the screen.
    pub fn build(self) -> Result<GuiElement, GuiError> {
        let queue = self.game_state.queue.clone();

        let width = self.dimensions.2;
        let height = self.dimensions.3;

        let mut image = image::RgbaImage::from_raw(
            width,
            height,
            vec![0; width as usize * height as usize * 4],
        )
        // only returns `None` if the given buffer isn't big enough for the requested dimensions.
        // Rgba is 4 bytes, and the dimensions are width * height, so the buffer should always be
        // big enough.
        .unwrap();

        for x in 0..width {
            for y in 0..height {
                let ps = if let Some(border_color) = is_border(x, y, width, height, self.border) {
                    border_color
                } else {
                    self.color
                };

                image.put_pixel(x, y, image::Rgba(ps));
            }
        }

        if let Some(request) = &self.text {
            let scale = rusttype::Scale::uniform(request.font_size as f32);
            let v_metrics = request.font.v_metrics(scale);
            let glyphs: Vec<_> = request
                .font
                .layout(
                    request.text.trim(),
                    scale,
                    rusttype::point(0.0, v_metrics.ascent),
                )
                .collect();

            if !glyphs.is_empty() {
                let total_bounding_box = calc_text_bounding_box(glyphs.iter());

                let text_width = total_bounding_box.max.x - total_bounding_box.min.x;
                let text_height = total_bounding_box.max.y - total_bounding_box.min.y;
                let position = (
                    (width as i32 - text_width) / 2,
                    (height as i32 - text_height) / 2,
                );
                let color = request.color;

                for glyph in glyphs {
                    if let Some(bounding_box) = glyph.pixel_bounding_box() {
                        glyph.draw(|x, y, v| {
                            let x = position.0 + x as i32 + bounding_box.min.x;
                            let y = position.1 + y as i32 + bounding_box.min.y;
                            if x < 0
                                || y < 0
                                || x >= image.width() as i32
                                || y >= image.height() as i32
                            {
                                return;
                            }
                            image.get_pixel_mut(x as u32, y as u32).blend(&image::Rgba([
                                color[0],
                                color[1],
                                color[2],
                                (v * 255.) as u8,
                            ]));
                        });
                    }
                }
            }
        }

        let (id, element_ref, element) = GuiElement::new(
            queue,
            self.dimensions,
            (width, height, image.into_raw()),
            self.game_state.internal_update_sender.clone(),
            Some(super::element::CanvasConfig {
                background: self.color,
                border: self.border,
                text: self.text,
            }),
        )?;
        self.game_state.gui_elements.insert(id, element_ref);

        Ok(element)
    }
}

fn calc_text_bounding_box<'a>(
    glyphs: impl Iterator<Item = &'a rusttype::PositionedGlyph<'a>>,
) -> rusttype::Rect<i32> {
    let mut total_bounding_box = rusttype::Rect {
        min: rusttype::Point {
            x: i32::max_value(),
            y: i32::max_value(),
        },
        max: rusttype::Point {
            x: i32::min_value(),
            y: i32::min_value(),
        },
    };

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            total_bounding_box.min.x = total_bounding_box.min.x.min(bounding_box.min.x);
            total_bounding_box.min.y = total_bounding_box.min.y.min(bounding_box.min.y);

            total_bounding_box.max.x = total_bounding_box.max.x.max(bounding_box.max.x);
            total_bounding_box.max.y = total_bounding_box.min.y.max(bounding_box.max.y);
        }
    }
    total_bounding_box
}

fn is_border(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    maybe_border: Option<(u16, [u8; 4])>,
) -> Option<[u8; 4]> {
    if let Some((border_width, border_color)) = maybe_border {
        let border_width = border_width as u32;
        if x < border_width
            || x + border_width >= width
            || y < border_width
            || y + border_width >= height
        {
            return Some(border_color);
        }
    }
    None
}
