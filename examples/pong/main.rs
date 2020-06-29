use cgmath::{Matrix4, Point3, Vector3};
use crystal_engine::*;

mod ball;
mod paddle;
mod score;

use self::{
    ball::{Ball, BallUpdate},
    paddle::Paddle,
    score::Score,
};

fn main() {
    Window::<Game>::new(800., 600.).unwrap().run();
}

pub struct Game {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    score: Score,
}

impl crystal_engine::Game for Game {
    fn init(state: &mut GameState) -> Self {
        let (left_paddle, right_paddle) = Paddle::new(state);

        state.camera = Matrix4::look_at(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        Self {
            left_paddle,
            right_paddle,
            ball: Ball::new(state),
            score: Score::new(state),
        }
    }

    fn update(&mut self, state: &mut GameState) {
        if state.keyboard.is_pressed(event::VirtualKeyCode::W) {
            self.left_paddle.up();
        }
        if state.keyboard.is_pressed(event::VirtualKeyCode::S) {
            self.left_paddle.down();
        }

        if state.keyboard.is_pressed(event::VirtualKeyCode::I) {
            self.right_paddle.up();
        }
        if state.keyboard.is_pressed(event::VirtualKeyCode::K) {
            self.right_paddle.down();
        }
        if state.keyboard.is_pressed(event::VirtualKeyCode::Space) {
            self.ball.start();
        }
        if state.keyboard.is_pressed(event::VirtualKeyCode::Escape) {
            state.terminate_game();
        }

        let result = self.ball.update(&self.left_paddle, &self.right_paddle);
        if let BallUpdate::Score { is_left } = result {
            self.score.update(is_left, state);
        }
    }
}
