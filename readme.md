# This project is on hold while I figure out what I want to do with this engine



This is a prototype game engine, focussed on abstracting away all rendering logic and focussing purely on the game logic.

# Example

```rust
use cgmath::{Deg, Euler, Matrix4, Point3, Rad, Vector3};
use crystal_engine::{
    event::VirtualKeyCode, DirectionalLight, GameState, LightColor, ModelHandle, Window,
};

fn main() {
    let window = Window::<Game>::new(800., 600.);
    window.run();
}

pub struct Game {
    rust_logo: ModelHandle,
    fbx_model: ModelHandle,
    obj_model: ModelHandle,
}

impl crystal_engine::Game for Game {
    fn init(state: &mut GameState) -> Self {
        state.window().set_title("crystal-engine demo");

        let rust_logo = state
            .new_rectangle_model()
            .with_texture_from_file("assets/rust_logo.png")
            .with_position((0.0, 1.5, 0.0))
            .with_rotation(Euler::new(Rad(0.0), Deg(180.0).into(), Rad(0.0)))
            .with_scale(1.0)
            .build();

        let fbx_model = state
            .new_fbx_model("assets/some_model.fbx")
            .build();

        let obj_model = state
            .new_obj_model("assets/some_model.obj")
            .build();

        state.camera = Matrix4::look_at(
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        state.light.directional.push(DirectionalLight {
            direction: Vector3::new(1.0, -1.0, 0.0),
            color: LightColor {
                ambient: Vector3::new(1.0, 1.0, 1.0),
                diffuse: Vector3::new(1.0, 1.0, 1.0),
                specular: Vector3::new(1.0, 1.0, 1.0),
            },
        });

        Self {
            rust_logo,
            fbx_model,
            obj_model,
        }
    }
    fn keydown(&mut self, state: &mut GameState, key: VirtualKeyCode) {
        if key == VirtualKeyCode::Escape {
            state.terminate_game();
        }
    }

    fn update(&mut self, _state: &mut GameState) {
        self.rust_logo.modify(|data| {
            data.rotation.y += Rad(0.02);
        });
    }
}
```


# Features
Currently the following features are available:

- **format-obj**: Allows loading .obj files, enabled by default.
- **format-fbx**: Allows loading .fbx binary files, enabled by default.


# Feedback
Feel free to approach me with any feedback you might have. You can open an issue on this repo, message me on twitter [@victorkoenders](https://twitter.com/victorkoenders) or on discord @Trangar#5901 
