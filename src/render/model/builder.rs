use super::{loader::SourceOrShape, Model, ModelHandle};
use crate::GameState;
use cgmath::{Euler, Rad, Vector3, Zero};
use parking_lot::RwLock;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, CommandBufferExecFuture},
    device::Queue,
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sync::NowFuture,
};

pub struct ModelBuilder<'a> {
    game_state: &'a mut GameState,
    source_or_shape: SourceOrShape<'a>,
    fallback_color: Option<Vector3<f32>>,
    texture: Option<&'a str>,
    position: Vector3<f32>,
    rotation: Euler<Rad<f32>>,
    scale: f32,
}

impl<'a> ModelBuilder<'a> {
    pub(crate) fn new(game_state: &'a mut GameState, source_or_shape: SourceOrShape<'a>) -> Self {
        Self {
            game_state,
            source_or_shape,
            fallback_color: None,
            texture: None,
            position: Vector3::zero(),
            rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
            scale: 1.0,
        }
    }

    pub fn with_fallback_color(mut self, color: impl Into<Vector3<f32>>) -> Self {
        self.fallback_color = Some(color.into());
        self
    }
    pub fn with_texture_from_file(mut self, texture_src: &'a str) -> Self {
        self.texture = Some(texture_src);
        self
    }
    pub fn with_position(mut self, position: impl Into<Vector3<f32>>) -> Self {
        self.position = position.into();
        self
    }
    pub fn with_rotation(mut self, rotation: Euler<Rad<f32>>) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn build(self) -> ModelHandle {
        let position = self.position;
        let rotation = self.rotation;
        let scale = self.scale;

        let (vertices, indices) = self.source_or_shape.into_vertices_and_indices();
        let device = self.game_state.device.clone();
        let indices = indices
            .into_iter()
            .map(|i| {
                CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    false,
                    i.into_iter().map(|i| *i),
                )
                .unwrap()
            })
            .collect();

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device,
            BufferUsage::all(),
            false,
            vertices.into_iter().map(|v| *v),
        )
        .unwrap();

        let mut model = Model {
            indices,
            vertex_buffer,
            texture: None,
            texture_future: RwLock::new(None),
        };

        if let Some(texture) = self.texture {
            let (tex, tex_future) = load_texture(self.game_state.queue.clone(), texture);
            model.texture = Some(tex);
            *model.texture_future.write() = Some(Box::new(tex_future) as _);
        }

        let (handle, id, data) =
            ModelHandle::from_model(Arc::new(model), self.game_state.model_handle_sender.clone());
        self.game_state.model_handles.insert(id, data);

        // TODO: Immediately set this on the handle
        handle.modify(|data| {
            data.position = position;
            data.rotation = rotation;
            data.scale = scale;
        });

        handle
    }
}

fn load_texture(
    queue: Arc<Queue>,
    path: &str,
) -> (
    Arc<ImmutableImage<R8G8B8A8Srgb>>,
    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
) {
    use std::fs::File;
    let file = File::open(path).unwrap();
    let decoder = png::Decoder::new(file);
    let (info, mut reader) = decoder.read_info().unwrap();
    let dimensions = Dimensions::Dim2d {
        width: info.width,
        height: info.height,
    };
    let mut image_data = Vec::new();
    image_data.resize((info.width * info.height * 4) as usize, 0);
    reader.next_frame(&mut image_data).unwrap();

    ImmutableImage::from_iter(image_data.iter().cloned(), dimensions, R8G8B8A8Srgb, queue).unwrap()
}
