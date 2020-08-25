use super::{
    handle::ModelRef, loader::SourceOrShape, Model, ModelDataGroup, ModelGroup, ModelHandle,
};
use crate::{error::ModelError, GameState, ModelData};
use parking_lot::RwLock;
use std::sync::Arc;
use vek::Vec3;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, CommandBufferExecFuture},
    device::Queue,
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sync::{GpuFuture, NowFuture},
};

/// A builder that is used to configure a model being loaded
pub struct ModelBuilder<'a> {
    game_state: &'a mut GameState,
    source_or_shape: SourceOrShape<'a>,
    fallback_color: Option<Vec3<f32>>,
    texture: Option<&'a str>,
    position: Vec3<f32>,
    rotation: Vec3<f32>,
    scale: f32,
}

impl<'a> ModelBuilder<'a> {
    pub(crate) fn new(game_state: &'a mut GameState, source_or_shape: SourceOrShape<'a>) -> Self {
        Self {
            game_state,
            source_or_shape,
            fallback_color: None,
            texture: None,
            position: Vec3::zero(),
            rotation: Vec3::zero(),
            scale: 1.0,
        }
    }

    /// Set the fallback color of the model in case the model has no texture
    pub fn with_fallback_color(mut self, color: impl Into<Vec3<f32>>) -> Self {
        self.fallback_color = Some(color.into());
        self
    }

    /// Set the texture to be used in this model
    pub fn with_texture_from_file(mut self, texture_src: &'a str) -> Self {
        self.texture = Some(texture_src);
        self
    }

    /// Set the initial position of the model
    pub fn with_position(mut self, position: impl Into<Vec3<f32>>) -> Self {
        self.position = position.into();
        self
    }

    /// Set the initial rotation of the model
    pub fn with_rotation(mut self, rotation: Vec3<f32>) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the initial scale of the model
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Finish configuring the model and try to load it.
    pub fn build(self) -> Result<ModelHandle, ModelError> {
        let position = self.position;
        let rotation = self.rotation;
        let scale = self.scale;

        let source = self.source_or_shape.parse()?;
        let device = self.game_state.device.clone();
        let queue = self.game_state.queue.clone();

        let (tex, mut futures) = if let Some(texture) = self.texture {
            let (tex, tex_future) = load_texture(self.game_state.queue.clone(), texture)?;
            (Some(tex), vec![tex_future.boxed()])
        } else {
            (None, Vec::new())
        };

        let vertex_buffer = if let Some(vertices) = source.vertices {
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                vertices.iter().copied(),
            )
            .ok()
        } else {
            None
        };

        let mut groups: Vec<_> = source
            .parts
            .into_iter()
            .map(|part| {
                let (group, maybe_future) =
                    ModelGroup::from_part(device.clone(), queue.clone(), &tex, part);
                if let Some(fut) = maybe_future {
                    futures.push(fut);
                }
                group
            })
            .collect();

        if groups.is_empty() {
            // we always need a single group, so add a dummy group
            // TODO: Why do we always need a single group?
            groups.push(ModelGroup::from_tex(tex));
        }

        let model = Model {
            vertex_buffer,
            groups,
            texture_future: RwLock::new(futures),
        };

        if model.vertex_buffer.is_none() && model.groups.iter().all(|g| g.vertex_buffer.is_none()) {
            return Err(ModelError::InvalidModelVertexBuffer);
        }

        let groups = (0..model.groups.len())
            .map(|_| ModelDataGroup::default())
            .collect();

        let (id, model_ref, model_handle) = ModelRef::new(
            Arc::new(model),
            self.game_state.internal_update_sender.clone(),
            ModelData {
                position,
                rotation,
                scale,
                groups,
            },
        );
        self.game_state.model_handles.insert(id, model_ref);

        Ok(model_handle)
    }
}

type LoadedTexture = (
    Arc<ImmutableImage<R8G8B8A8Srgb>>,
    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
);

fn load_texture(queue: Arc<Queue>, path: &str) -> Result<LoadedTexture, ModelError> {
    let image = image::open(path)
        .map_err(|inner| ModelError::CouldNotLoadTexture {
            path: path.to_owned(),
            inner,
        })?
        .to_rgba();
    let dimensions = Dimensions::Dim2d {
        width: image.width(),
        height: image.height(),
    };

    Ok(ImmutableImage::from_iter(
        image.into_raw().into_iter(),
        dimensions,
        R8G8B8A8Srgb,
        queue,
    )
    // Should never fail because the image is in the correct format, the dimensions
    // match and the queue is assumed to be valid
    .unwrap())
}
