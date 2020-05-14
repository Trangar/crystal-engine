This is a prototype game engine, focussed on abstracting away all rendering logic and focussing purely on the game logic.

# Example

```rust
use cgmath::{Matrix4, Point3, Rad, Vector3};
use crystal_engine::{GameState, ModelHandle, Window, VirtualKeyCode};

fn main() {
    // Create a new instance of your game and run it
    let window = Window::<Game>::new(800., 600.);
    window.run();
}

pub struct Game {
    // Your game state is stored here
    model: ModelHandle,
}

impl crystal_engine::Game for Game {
    fn init(state: &mut GameState) -> Self {
        // Load an object. This will automatically be rendered every frame
        // as long as the returned ModelHandle is not dropped.
        let model = state.create_model_from_obj("assets/some_object.obj");

        // You can move the model around by calling `.modify`
        model.modify(|data| {
            data.position.y = -3.0;
            data.scale = 0.3;
        });

        // Update the camera by manipulating the state's field
        state.camera = Matrix4::look_at(
            Point3::new(0.3, 0.3, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );

        Self { model }
    }

    fn keydown(&mut self, state: &mut GameState, key: VirtualKeyCode) {
        // Exit the game when the user hits escape
        if key == VirtualKeyCode::Escape {
            state.terminate_game();
        }
    }

    fn update(&mut self, state: &mut GameState) {
        self.model.modify(|data| {
            // Rotate either left or right, based on what the user has pressed
            if state.keyboard.is_pressed(VirtualKeyCode::A) {
                data.rotation.y -= Rad(0.05);
            }
            if state.keyboard.is_pressed(VirtualKeyCode::D) {
                data.rotation.y += Rad(0.05);
            }
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
