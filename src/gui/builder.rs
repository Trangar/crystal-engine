use super::GuiElement;
use crate::GameState;
use glyph_brush::{
    ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text,
    VerticalAlign,
};
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
    font: FontArc,
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
        font: FontArc,
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

        for x in 0..width {
            for y in 0..height {
                let ps = if let Some(border_color) = is_border(x, y, width, height, &self.border) {
                    border_color
                } else {
                    &self.color
                };

                let idx = ((y as usize * width as usize) + x as usize) * 4;
                data[idx..idx + 4].copy_from_slice(ps);
            }
        }

        if let Some(request) = self.text {
            let mut glyph_brush: GlyphBrush<()> =
                GlyphBrushBuilder::using_font(request.font).build();

            glyph_brush.queue(
                Section::default()
                    .add_text(Text::new(&request.text).with_scale(request.font_size as f32))
                    .with_bounds((width as f32, height as f32))
                    .with_layout(
                        Layout::default()
                            .h_align(HorizontalAlign::Center)
                            .v_align(VerticalAlign::Center),
                    ),
            );

            let font_color = request.color;

            glyph_brush
                .process_queued(
                    |rect, mut px| {
                        for x in rect.min[0]..rect.max[0] {
                            for y in rect.min[1]..rect.max[1] {
                                let idx = ((y as usize * width as usize) + x as usize) * 4;
                                let (alpha, remaining) = px.split_first().unwrap();
                                px = remaining;

                                let color = &mut data[idx..idx + 4];
                                // TODO: Properly merge these colors
                                if *alpha > 100 {
                                    color.copy_from_slice(&font_color);
                                }
                            }
                        }
                        assert!(px.is_empty());
                    },
                    |_vertex_data| {},
                )
                .unwrap();
        }

        let (element_ref, element) = GuiElement::new(queue, self.dimensions, (width, height, data));
        self.game_state.gui_elements.push(element_ref);

        element
    }
}

fn is_border(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    maybe_border: &Option<(u16, [u8; 4])>,
) -> Option<&[u8; 4]> {
    if let Some((border_width, border_color)) = maybe_border {
        let border_width = *border_width as u32;
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
