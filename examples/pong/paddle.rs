use crystal_engine::*;
use vek::{Vec2, Vec3};

pub struct Paddle {
    pub position: Vec2<f32>,
    handle: ModelHandle,
}

impl Paddle {
    pub fn new(state: &mut GameState) -> (Self, Self) {
        let handle = state
            .new_obj_model("examples/pong/assets/paddle.obj")
            .with_rotation(Vec3::new(90.0, 0.0, 0.0))
            .build()
            .unwrap();
        let left = Paddle {
            position: Vec2::new(-1.0, 0.0),
            handle: handle.clone(),
        };
        left.update_position();

        let right = Paddle {
            position: Vec2::new(1.0, 0.0),
            handle,
        };
        right.update_position();

        (left, right)
    }

    fn update_position(&self) {
        self.handle
            .modify(|d| d.position = Vec3::from_direction_2d(self.position));
    }

    pub fn up(&mut self) {
        if self.position.y < 1.0 {
            self.position.y += 0.1;
            self.update_position();
        }
    }

    pub fn down(&mut self) {
        if self.position.y > -1.0 {
            self.position.y -= 0.1;
            self.update_position();
        }
    }
}
