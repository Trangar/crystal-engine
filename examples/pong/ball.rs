use crate::Paddle;
use crystal_engine::*;
use rand::{thread_rng, Rng};
use vek::{Vec2, Vec3};

pub struct Ball {
    position: Vec2<f32>,
    direction: Vec2<f32>,
    handle: ModelHandle,
}

impl Ball {
    pub fn new(state: &mut GameState) -> Self {
        Self {
            position: Vec2::zero(),
            direction: Vec2::zero(),
            handle: state
                .new_obj_model("examples/pong/assets/ball.obj")
                .build()
                .unwrap(),
        }
    }

    fn hits(&self, paddle: &Paddle) -> bool {
        let diff = self.position - paddle.position;

        diff.x.abs() < 0.2 && diff.y.abs() < 0.3
    }

    pub fn start(&mut self) {
        if self.direction.magnitude_squared() < std::f32::EPSILON {
            let mut rng = thread_rng();
            let x = if rng.gen::<bool>() { -1.0 } else { 1.0 };
            let y = if rng.gen::<bool>() { -1.0 } else { 1.0 };
            self.direction = Vec2::new(x, y);
        }
    }

    fn reset(&mut self) {
        self.position = Vec2::zero();
        self.direction = Vec2::zero();
    }

    pub fn update(&mut self, left_paddle: &Paddle, right_paddle: &Paddle) -> BallUpdate {
        if self.direction.x < 0. {
            if self.hits(left_paddle) {
                self.direction.x *= -1.01;
            } else if self.position.x < -1.2 {
                self.reset();
                return BallUpdate::Score { is_left: false };
            }
        } else {
            // moving right
            if self.hits(right_paddle) {
                self.direction.x *= -1.01;
            } else if self.position.x > 1.2 {
                self.reset();
                return BallUpdate::Score { is_left: true };
            }
        }

        if (self.position.y > 1.0 && self.direction.y > 0.)
            || (self.position.y < -1.0 && self.direction.y < 0.)
        {
            self.direction.y *= -1.0;
        }

        self.position += self.direction / 50.;

        self.handle
            .modify(|d| d.position = Vec3::from_direction_2d(self.position));
        BallUpdate::None
    }
}

pub enum BallUpdate {
    Score { is_left: bool },
    None,
}
