use super::GuiElement;
use crate::GameState;
use image::Pixel;
use rusttype::Font;
use std::borrow::Cow;

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

    pub fn with_texture<'b>(self, texture_path: &'b str) -> GuiElementTextureBuilder<'a, 'b> {
        GuiElementTextureBuilder {
            game_state: self.game_state,
            dimensions: self.dimensions,
            texture_path,
        }
    }

    pub fn with_canvas(self, background_color: [u8; 4]) -> GuiElementCanvasBuilder<'a, 'static> {
        GuiElementCanvasBuilder {
            game_state: self.game_state,
            dimensions: self.dimensions,
            color: background_color,
            text: None,
            border: None,
        }
    }
}

pub struct GuiElementTextureBuilder<'a, 'b> {
    game_state: &'a mut GameState,
    dimensions: (i32, i32, u32, u32),
    texture_path: &'b str,
}
impl<'a, 'b> GuiElementTextureBuilder<'a, 'b> {
    pub fn build(self) -> GuiElement {
        let queue = self.game_state.queue.clone();
        let image = image::open(self.texture_path).unwrap().to_rgba();

        let (element_ref, element) = GuiElement::new(
            queue,
            self.dimensions,
            (image.width(), image.height(), image.into_raw()),
        );
        self.game_state.gui_elements.push(element_ref);

        element
    }
}
pub struct GuiElementCanvasBuilder<'a, 'b> {
    game_state: &'a mut GameState,
    dimensions: (i32, i32, u32, u32),
    color: [u8; 4],
    text: Option<TextRequest<'b>>,
    border: Option<(u16, [u8; 4])>,
}

struct TextRequest<'a> {
    font: &'a Font<'a>,
    font_size: u16,
    text: Cow<'a, str>,
    color: [u8; 4],
}

impl<'a, 'b> GuiElementCanvasBuilder<'a, 'b> {
    pub fn with_border(mut self, border_width: u16, border_color: [u8; 4]) -> Self {
        self.border = Some((border_width, border_color));
        self
    }
    pub fn with_text(
        mut self,
        font: &'b Font<'b>,
        font_size: u16,
        text: Cow<'b, str>,
        color: [u8; 4],
    ) -> Self {
        self.text = Some(TextRequest {
            font,
            font_size,
            text,
            color,
        });
        self
    }

    pub fn build(self) -> GuiElement {
        let queue = self.game_state.queue.clone();

        let width = self.dimensions.2;
        let height = self.dimensions.3;

        let mut data = vec![0; width as usize * height as usize * 4];

        let mut image = image::RgbaImage::from_raw(width, height, data).unwrap();

        for x in 0..width {
            for y in 0..height {
                let ps = if let Some(border_color) = is_border(x, y, width, height, &self.border) {
                    border_color
                } else {
                    self.color
                };

                image.put_pixel(x, y, image::Rgba(ps));
            }
        }

        if let Some(request) = self.text {
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

        let (element_ref, element) =
            GuiElement::new(queue, self.dimensions, (width, height, image.into_raw()));
        self.game_state.gui_elements.push(element_ref);

        element
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
    maybe_border: &Option<(u16, [u8; 4])>,
) -> Option<[u8; 4]> {
    if let Some((border_width, border_color)) = maybe_border {
        let border_width = *border_width as u32;
        if x < border_width
            || x + border_width >= width
            || y < border_width
            || y + border_width >= height
        {
            return Some(*border_color);
        }
    }
    None
}
