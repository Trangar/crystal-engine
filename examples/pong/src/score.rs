use crystal_engine::*;

pub struct Score {
    font: Font<'static>,
    left: u32,
    left_element: GuiElement,
    right: u32,
    right_element: GuiElement,
}

impl Score {
    const LEFT_POSITION: (i32, i32, u32, u32) = (100, 500, 100, 100);
    const RIGHT_POSITION: (i32, i32, u32, u32) = (600, 500, 100, 100);
    const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
    const WHITE: [u8; 4] = [255, 255, 255, 255];

    pub fn new(state: &mut GameState) -> Self {
        let font = state.load_font("assets/roboto.ttf").unwrap();
        Self {
            left: 0,
            left_element: Self::score_label(state, &font, Self::LEFT_POSITION, 0),
            right: 0,
            right_element: Self::score_label(state, &font, Self::RIGHT_POSITION, 0),
            font,
        }
    }

    pub fn update(&mut self, is_left: bool, state: &mut GameState) {
        let (new_value, position) = if is_left {
            self.left += 1;
            (self.left, Self::LEFT_POSITION)
        } else {
            self.right += 1;
            (self.right, Self::RIGHT_POSITION)
        };

        let new_element = Self::score_label(state, &self.font, position, new_value);

        if is_left {
            self.left_element = new_element;
        } else {
            self.right_element = new_element;
        }
    }

    fn score_label(
        state: &mut GameState,
        font: &Font,
        at: (i32, i32, u32, u32),
        count: u32,
    ) -> GuiElement {
        state
            .new_gui_element(at)
            .with_canvas(Self::TRANSPARENT)
            .with_text(font, 32, count.to_string().into(), Self::WHITE)
            .build()
            .unwrap()
    }
}
