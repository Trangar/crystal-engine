use crystal_engine::*;

pub struct Score {
    left: u32,
    left_element: GuiElement,
    right: u32,
    right_element: GuiElement,
}

impl Score {
    const LEFT_POSITION: (i32, i32, u32, u32) = (100, 500, 100, 100);
    const RIGHT_POSITION: (i32, i32, u32, u32) = (600, 500, 100, 100);

    pub fn new(state: &mut GameState) -> Self {
        let font = state.load_font("examples/pong/assets/roboto.ttf").unwrap();
        Self {
            left: 0,
            left_element: state
                .new_gui_element(Self::LEFT_POSITION)
                .canvas()
                .with_text(font.clone(), 32, "0", color::WHITE)
                .build()
                .unwrap(),
            right: 0,
            right_element: state
                .new_gui_element(Self::RIGHT_POSITION)
                .canvas()
                .with_text(font.clone(), 32, "0", color::WHITE)
                .build()
                .unwrap(),
        }
    }

    pub fn update(&mut self, is_left: bool, state: &mut GameState) {
        let (element, score) = if is_left {
            self.left += 1;
            (&mut self.left_element, self.left)
        } else {
            self.right += 1;
            (&mut self.right_element, self.right)
        };

        element
            .update_canvas(state, |b| b.with_text_content(score))
            .unwrap();
    }
}
