use cgmath::{Deg, Euler, Rad, Vector2};
use crystal_engine::*;

pub struct Paddle {
    pub position: Vector2<f32>,
    handle: ModelHandle,
}

impl Paddle {
    pub fn new(state: &mut GameState) -> (Self, Self) {
        let handle = state
            .new_obj_model("examples/pong/assets/paddle.obj")
            .with_rotation(Euler::new(Deg(90.0).into(), Rad(0.0), Rad(0.0)))
            .build()
            .unwrap();
        let left = Paddle {
            position: Vector2::new(-1.0, 0.0),
            handle: handle.clone(),
        };
        left.update_position();

        let right = Paddle {
            position: Vector2::new(1.0, 0.0),
            handle,
        };
        right.update_position();

        (left, right)
    }

    fn update_position(&self) {
        self.handle
            .modify(|d| d.position = self.position.extend(0.0));
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
